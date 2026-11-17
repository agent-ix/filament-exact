// SPDX-License-Identifier: AGPL-3.0-or-later
//! Verus proofs for the fixed-width canonical-fraction helpers (FR-408
//! Behavior 18 row a, IR-1544).
//!
//! The file is compiled only by a Verus run (`cfg(verus_keep_ghost)`). It holds
//! the mathematics (divisibility, Euclid's gcd) and, for every production
//! function of `native_fraction`, a specification function of the same name
//! that the production function is proved equal to.

use vstd::arithmetic::div_mod::*;
use vstd::arithmetic::mul::*;
use vstd::arithmetic::power2::*;
use vstd::prelude::*;

verus! {

// ---- divisibility -----------------------------------------------------------

/// `d` divides `n`.
pub open spec fn divides(d: nat, n: nat) -> bool {
    d != 0 && n % d == 0
}

/// `d` divides both `a` and `b`.
pub open spec fn common(d: nat, a: nat, b: nat) -> bool {
    divides(d, a) && divides(d, b)
}

/// `(a, b)` and `(c, e)` have the same common divisors.
pub open spec fn same_common(a: nat, b: nat, c: nat, e: nat) -> bool {
    forall|d: nat| #![trigger common(d, a, b)] #![trigger common(d, c, e)] common(d, a, b) <==> common(d, c, e)
}

/// The greatest common divisor by Euclid's algorithm; `gcd(0, 0)` is 0.
pub open spec fn gcd(a: nat, b: nat) -> nat
    decreases b,
{
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub proof fn lemma_divides_witness(d: nat, n: nat)
    requires
        divides(d, n),
    ensures
        n == d * (n / d),
{
    lemma_fundamental_div_mod(n as int, d as int);
}

pub proof fn lemma_divides_intro(d: nat, k: nat)
    requires
        d != 0,
    ensures
        divides(d, d * k),
{
    lemma_mod_multiples_basic(k as int, d as int);
    lemma_mul_is_commutative(d as int, k as int);
}

pub proof fn lemma_divides_le(d: nat, n: nat)
    requires
        divides(d, n),
        n != 0,
    ensures
        d <= n,
{
    lemma_mod_is_zero(n, d);
}

pub proof fn lemma_divides_add(d: nat, x: nat, y: nat)
    requires
        divides(d, x),
        divides(d, y),
    ensures
        divides(d, x + y),
{
    lemma_add_mod_noop(x as int, y as int, d as int);
    lemma_small_mod(0, d);
}

pub proof fn lemma_divides_sub(d: nat, x: nat, y: nat)
    requires
        divides(d, x),
        divides(d, y),
        x >= y,
    ensures
        divides(d, (x - y) as nat),
{
    lemma_sub_mod_noop(x as int, y as int, d as int);
    lemma_small_mod(0, d);
}

pub proof fn lemma_divides_mul(d: nat, x: nat, k: nat)
    requires
        divides(d, x),
    ensures
        divides(d, x * k),
        divides(d, k * x),
{
    lemma_divides_witness(d, x);
    let q = x / d;
    assert(x * k == d * (q * k)) by (nonlinear_arith)
        requires
            x == d * q,
    ;
    lemma_divides_intro(d, q * k);
    lemma_mul_is_commutative(x as int, k as int);
}

pub proof fn lemma_divides_trans(a: nat, b: nat, c: nat)
    requires
        divides(a, b),
        divides(b, c),
    ensures
        divides(a, c),
{
    lemma_divides_witness(b, c);
    lemma_divides_mul(a, b, c / b);
}

pub proof fn lemma_divides_antisym(a: nat, b: nat)
    requires
        divides(a, b),
        divides(b, a),
    ensures
        a == b,
{
    lemma_divides_le(a, b);
    lemma_divides_le(b, a);
}

// ---- Euclid's gcd is the greatest common divisor -----------------------------

pub proof fn lemma_gcd_common(a: nat, b: nat)
    requires
        a != 0 || b != 0,
    ensures
        common(gcd(a, b), a, b),
    decreases b,
{
    if b == 0 {
        lemma_mod_self_0(a as int);
        lemma_small_mod(0, a);
    } else {
        lemma_gcd_common(b, a % b);
        let g = gcd(b, a % b);
        lemma_fundamental_div_mod(a as int, b as int);
        lemma_divides_mul(g, b, a / b);
        lemma_divides_add(g, b * (a / b), a % b);
    }
}

pub proof fn lemma_gcd_greatest(a: nat, b: nat, d: nat)
    requires
        common(d, a, b),
    ensures
        divides(d, gcd(a, b)),
    decreases b,
{
    if b != 0 {
        lemma_fundamental_div_mod(a as int, b as int);
        lemma_divides_mul(d, b, a / b);
        lemma_divides_sub(d, a, b * (a / b));
        lemma_gcd_greatest(b, a % b, d);
    }
}

pub proof fn lemma_gcd_unique(a: nat, b: nat, g: nat)
    requires
        a != 0 || b != 0,
        common(g, a, b),
        forall|d: nat| #[trigger] common(d, a, b) ==> divides(d, g),
    ensures
        g == gcd(a, b),
{
    lemma_gcd_common(a, b);
    lemma_gcd_greatest(a, b, g);
    lemma_divides_antisym(g, gcd(a, b));
}

pub proof fn lemma_same_common_gcd(a: nat, b: nat, c: nat, e: nat)
    requires
        same_common(a, b, c, e),
        a != 0 || b != 0,
        c != 0 || e != 0,
    ensures
        gcd(a, b) == gcd(c, e),
{
    lemma_gcd_common(a, b);
    lemma_gcd_common(c, e);
    let g = gcd(a, b);
    assert(common(g, c, e));
    assert forall|d: nat| #[trigger] common(d, c, e) implies divides(d, g) by {
        assert(common(d, a, b));
        lemma_gcd_greatest(a, b, d);
    }
    lemma_gcd_unique(c, e, g);
}

// ---- odd values and powers of two -------------------------------------------

/// `n` is odd.
pub open spec fn odd(n: nat) -> bool {
    n % 2 == 1
}

/// A pair the binary gcd steps work on: an odd value and an odd value or zero.
pub open spec fn odd_state(a: nat, b: nat) -> bool {
    odd(a) && (b == 0 || odd(b))
}

pub proof fn lemma_odd_divisor(d: nat, s: nat)
    requires
        divides(d, s),
        odd(s),
    ensures
        odd(d),
{
    lemma_divides_witness(d, s);
    lemma_mul_mod_noop_general(d as int, (s / d) as int, 2);
    if d % 2 == 0 {
        assert(((d % 2) * ((s / d) % 2)) == 0) by (nonlinear_arith)
            requires
                d % 2 == 0,
        ;
    }
}

pub proof fn lemma_odd_divides_half(d: nat, m: nat)
    requires
        odd(d),
        divides(d, 2 * m),
    ensures
        divides(d, m),
{
    let k = (2 * m) / d;
    lemma_divides_witness(d, 2 * m);
    lemma_mul_mod_noop_general(d as int, k as int, 2);
    assert(((d % 2) * (k % 2)) == k % 2) by (nonlinear_arith)
        requires
            d % 2 == 1,
    ;
    assert((2 * m) % 2 == 0) by (nonlinear_arith);
    assert(k % 2 == 0);
    let j = k / 2;
    assert(k == 2 * j);
    assert(m == d * j) by (nonlinear_arith)
        requires
            2 * m == d * k,
            k == 2 * j,
    ;
    lemma_divides_intro(d, j);
}

pub proof fn lemma_odd_divides_pow2(d: nat, m: nat, t: nat)
    requires
        odd(d),
        divides(d, m * pow2(t)),
    ensures
        divides(d, m),
    decreases t,
{
    if t == 0 {
        lemma2_to64();
        assert(m * pow2(0) == m) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
    } else {
        lemma_pow2_unfold(t);
        let h = pow2((t - 1) as nat);
        assert(m * pow2(t) == 2 * (m * h)) by (nonlinear_arith)
            requires
                pow2(t) == 2 * h,
        ;
        lemma_odd_divides_half(d, m * h);
        lemma_odd_divides_pow2(d, m, (t - 1) as nat);
    }
}

/// An odd `a` has the same common divisors with `b * 2^t` as with `b`.
pub proof fn lemma_strip_pow2(a: nat, b: nat, t: nat)
    requires
        odd(a),
    ensures
        same_common(a, b * pow2(t), a, b),
{
    assert forall|d: nat|
        #![trigger common(d, a, b * pow2(t))]
        #![trigger common(d, a, b)]
        common(d, a, b * pow2(t)) <==> common(d, a, b) by {
        if common(d, a, b * pow2(t)) {
            lemma_odd_divisor(d, a);
            lemma_odd_divides_pow2(d, b, t);
        }
        if common(d, a, b) {
            lemma_divides_mul(d, b, pow2(t));
        }
    }
}

/// The mathematics of one binary gcd step on two distinct odd values.
pub proof fn lemma_step_math(small: nat, large: nat, new: nat, t: nat)
    requires
        odd(small),
        odd(large),
        large > small,
        large - small == new * pow2(t),
        odd(new),
    ensures
        same_common(small, new, small, large),
        2 * (small * new) <= small * large,
{
    lemma_pow2_pos(t);
    if t == 0 {
        lemma2_to64();
        assert(new * pow2(0) == new) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
        assert(false);
    }
    lemma_pow2_unfold(t);
    let h = pow2((t - 1) as nat);
    lemma_pow2_pos((t - 1) as nat);
    assert(2 * new <= new * pow2(t)) by (nonlinear_arith)
        requires
            pow2(t) == 2 * h,
            h >= 1,
    ;
    assert(2 * (small * new) <= small * large) by (nonlinear_arith)
        requires
            2 * new <= large - small,
            large > small,
    ;
    let diff = (large - small) as nat;
    lemma_strip_pow2(small, new, t);
    assert forall|d: nat|
        #![trigger common(d, small, new)]
        #![trigger common(d, small, large)]
        common(d, small, new) <==> common(d, small, large) by {
        assert(common(d, small, new) <==> common(d, small, new * pow2(t)));
        if common(d, small, diff) {
            lemma_divides_add(d, diff, small);
        }
        if common(d, small, large) {
            lemma_divides_sub(d, large, small);
        }
    }
}

// ---- scaling and symmetry of the gcd ------------------------------------------

pub proof fn lemma_gcd_zero_left(b: nat)
    ensures
        gcd(0, b) == b,
{
    if b != 0 {
        lemma_small_mod(0, b);
        reveal_with_fuel(gcd, 3);
    }
}

pub proof fn lemma_same_common_swap(a: nat, b: nat)
    ensures
        same_common(a, b, b, a),
{
    assert forall|d: nat|
        #![trigger common(d, a, b)]
        #![trigger common(d, b, a)]
        common(d, a, b) <==> common(d, b, a) by {}
}

pub proof fn lemma_gcd_sym(a: nat, b: nat)
    ensures
        gcd(a, b) == gcd(b, a),
{
    if a == 0 && b == 0 {
    } else {
        lemma_same_common_swap(a, b);
        lemma_same_common_gcd(a, b, b, a);
    }
}

/// `gcd(k x, k y) == k gcd(x, y)`.
pub proof fn lemma_gcd_scale(k: nat, x: nat, y: nat)
    requires
        k != 0,
    ensures
        gcd(k * x, k * y) == k * gcd(x, y),
    decreases y,
{
    if y == 0 {
        assert(k * y == 0) by (nonlinear_arith)
            requires
                y == 0,
        ;
    } else {
        assert(k * y != 0) by (nonlinear_arith)
            requires
                k != 0,
                y != 0,
        ;
        lemma_truncate_middle(x as int, k as int, y as int);
        lemma_gcd_scale(k, y, x % y);
    }
}

/// An odd `b` and `a * 2^t` have the gcd of `a` and `b`.
pub proof fn lemma_gcd_strip_pow2(a: nat, b: nat, t: nat)
    requires
        odd(b),
    ensures
        gcd(a * pow2(t), b) == gcd(a, b),
{
    lemma_strip_pow2(b, a, t);
    assert forall|d: nat|
        #![trigger common(d, a * pow2(t), b)]
        #![trigger common(d, a, b)]
        common(d, a * pow2(t), b) <==> common(d, a, b) by {
        assert(common(d, b, a * pow2(t)) <==> common(d, b, a));
    }
    lemma_same_common_gcd(a * pow2(t), b, a, b);
}

/// The gcd of two odd parts scaled by powers of two.
pub proof fn lemma_gcd_odd_parts(lo: nat, ro: nat, tl: nat, tr: nat)
    requires
        odd(lo),
        odd(ro),
    ensures
        gcd(lo * pow2(tl), ro * pow2(tr)) == gcd(lo, ro) * pow2(if tl <= tr { tl } else { tr }),
{
    if tl <= tr {
        let e = (tr - tl) as nat;
        lemma_pow2_adds(tl, e);
        lemma_pow2_pos(tl);
        let k = pow2(tl);
        assert(ro * pow2(tr) == k * (ro * pow2(e))) by (nonlinear_arith)
            requires
                pow2(tr) == k * pow2(e),
        ;
        assert(lo * pow2(tl) == k * lo) by (nonlinear_arith)
            requires
                k == pow2(tl),
        ;
        lemma_gcd_scale(k, lo, ro * pow2(e));
        lemma_gcd_sym(lo, ro * pow2(e));
        lemma_gcd_strip_pow2(ro, lo, e);
        lemma_gcd_sym(ro, lo);
        assert(k * gcd(lo, ro) == gcd(lo, ro) * k) by (nonlinear_arith);
    } else {
        let e = (tl - tr) as nat;
        lemma_pow2_adds(tr, e);
        lemma_pow2_pos(tr);
        let k = pow2(tr);
        assert(lo * pow2(tl) == k * (lo * pow2(e))) by (nonlinear_arith)
            requires
                pow2(tl) == k * pow2(e),
        ;
        assert(ro * pow2(tr) == k * ro) by (nonlinear_arith)
            requires
                k == pow2(tr),
        ;
        lemma_gcd_scale(k, lo * pow2(e), ro);
        lemma_gcd_strip_pow2(lo, ro, e);
        assert(k * gcd(lo, ro) == gcd(lo, ro) * k) by (nonlinear_arith);
    }
}

// ---- the 128-step u64 ladder --------------------------------------------------

/// Specification of the production `step`.
pub open spec fn step(s: (u64, u64)) -> (u64, u64) {
    let (left, right) = s;
    if right == 0 {
        (left, right)
    } else {
        let (small, large) = if left > right {
            (right, left)
        } else {
            (left, right)
        };
        let difference = (large - small) as u64;
        if difference == 0 {
            (small, 0u64)
        } else {
            (small, difference >> (difference.trailing_zeros() as u64))
        }
    }
}

pub open spec fn steps_2(s: (u64, u64)) -> (u64, u64) {
    step(step(s))
}

pub open spec fn steps_4(s: (u64, u64)) -> (u64, u64) {
    steps_2(steps_2(s))
}

pub open spec fn steps_8(s: (u64, u64)) -> (u64, u64) {
    steps_4(steps_4(s))
}

pub open spec fn steps_16(s: (u64, u64)) -> (u64, u64) {
    steps_8(steps_8(s))
}

pub open spec fn steps_32(s: (u64, u64)) -> (u64, u64) {
    steps_16(steps_16(s))
}

pub open spec fn steps_64(s: (u64, u64)) -> (u64, u64) {
    steps_32(steps_32(s))
}

#[verifier::opaque]
pub open spec fn steps_128(s: (u64, u64)) -> (u64, u64) {
    steps_64(steps_64(s))
}

/// `k` steps.
pub open spec fn iter(s: (u64, u64), k: nat) -> (u64, u64)
    decreases k,
{
    if k == 0 {
        s
    } else {
        iter(step(s), (k - 1) as nat)
    }
}

/// A nonzero `u64` is its odd part times a power of two.
pub proof fn lemma_u64_odd_part(x: u64)
    requires
        x != 0,
    ensures
        x.trailing_zeros() < 64,
        x as nat == ((x >> (x.trailing_zeros() as u64)) as nat) * pow2(x.trailing_zeros() as nat),
        odd((x >> (x.trailing_zeros() as u64)) as nat),
{
    vstd::std_specs::bits::axiom_u64_trailing_zeros(x);
    let t = x.trailing_zeros() as u64;
    let y = x >> t;
    assert(y % 2 == 1) by (bit_vector)
        requires
            y & 1 == 1,
    ;
    assert((y << t) == x) by (bit_vector)
        requires
            t < 64,
            y == x >> t,
            x << sub(64, t) == 0,
    ;
    vstd::bits::lemma_u64_shr_is_div(x, t);
    let p = pow2(t as nat);
    lemma_pow2_pos(t as nat);
    lemma_fundamental_div_mod(x as int, p as int);
    lemma_mod_bound(x as int, p as int);
    assert(y * p <= x) by (nonlinear_arith)
        requires
            x == p * (x as nat / p) + (x as nat % p),
            y == x as nat / p,
            (x as nat % p) >= 0,
    ;
    vstd::bits::lemma_u64_shl_is_mul(y, t);
}

pub proof fn lemma_step(s: (u64, u64))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        odd_state(step(s).0 as nat, step(s).1 as nat),
        same_common(step(s).0 as nat, step(s).1 as nat, s.0 as nat, s.1 as nat),
        s.1 != 0 ==> 2 * (step(s).0 as nat * step(s).1 as nat) <= s.0 as nat * s.1 as nat,
        s.1 == 0 ==> step(s) == s,
{
    let (left, right) = s;
    if right != 0 {
        let (small, large) = if left > right {
            (right, left)
        } else {
            (left, right)
        };
        let difference = (large - small) as u64;
        if difference == 0 {
            assert forall|d: nat|
                #![trigger common(d, small as nat, 0)]
                #![trigger common(d, left as nat, right as nat)]
                common(d, small as nat, 0) <==> common(d, left as nat, right as nat) by {
                if d != 0 {
                    lemma_small_mod(0, d);
                }
            }
            assert(small as nat * 0 == 0) by (nonlinear_arith);
        } else {
            lemma_u64_odd_part(difference);
            let t = difference.trailing_zeros() as nat;
            let new = difference >> (difference.trailing_zeros() as u64);
            lemma_step_math(small as nat, large as nat, new as nat, t);
            assert forall|d: nat|
                #![trigger common(d, small as nat, new as nat)]
                #![trigger common(d, left as nat, right as nat)]
                common(d, small as nat, new as nat) <==> common(d, left as nat, right as nat) by {
                assert(common(d, small as nat, new as nat) <==> common(
                    d,
                    small as nat,
                    large as nat,
                ));
            }
            assert(small as nat * large as nat == left as nat * right as nat) by (nonlinear_arith)
                requires
                    (small == left && large == right) || (small == right && large == left),
            ;
        }
    } else {
        assert forall|d: nat|
            #![trigger common(d, left as nat, right as nat)]
            common(d, left as nat, right as nat) <==> common(d, left as nat, right as nat) by {}
    }
}

pub proof fn lemma_iter_add(s: (u64, u64), a: nat, b: nat)
    ensures
        iter(iter(s, a), b) == iter(s, a + b),
    decreases a,
{
    if a != 0 {
        lemma_iter_add(step(s), (a - 1) as nat, b);
    }
}

pub proof fn lemma_iter(s: (u64, u64), k: nat)
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        odd_state(iter(s, k).0 as nat, iter(s, k).1 as nat),
        same_common(iter(s, k).0 as nat, iter(s, k).1 as nat, s.0 as nat, s.1 as nat),
        (iter(s, k).0 as nat * iter(s, k).1 as nat) * pow2(k) <= s.0 as nat * s.1 as nat,
    decreases k,
{
    if k == 0 {
        lemma2_to64();
        assert((s.0 as nat * s.1 as nat) * pow2(0) == s.0 as nat * s.1 as nat) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
        assert forall|d: nat|
            #![trigger common(d, s.0 as nat, s.1 as nat)]
            common(d, s.0 as nat, s.1 as nat) <==> common(d, s.0 as nat, s.1 as nat) by {}
    } else {
        let s1 = step(s);
        lemma_step(s);
        lemma_iter(s1, (k - 1) as nat);
        let r = iter(s, k);
        let m = r.0 as nat * r.1 as nat;
        let m1 = s1.0 as nat * s1.1 as nat;
        let m0 = s.0 as nat * s.1 as nat;
        let h = pow2((k - 1) as nat);
        lemma_pow2_unfold(k);
        assert(m * pow2(k) <= m0) by (nonlinear_arith)
            requires
                m * h <= m1,
                pow2(k) == 2 * h,
                (2 * m1 <= m0) || (m1 == m0 && m0 == 0),
                h >= 0,
        ;
        assert forall|d: nat|
            #![trigger common(d, r.0 as nat, r.1 as nat)]
            #![trigger common(d, s.0 as nat, s.1 as nat)]
            common(d, r.0 as nat, r.1 as nat) <==> common(d, s.0 as nat, s.1 as nat) by {
            assert(common(d, r.0 as nat, r.1 as nat) <==> common(d, s1.0 as nat, s1.1 as nat));
            assert(common(d, s1.0 as nat, s1.1 as nat) <==> common(d, s.0 as nat, s.1 as nat));
        }
    }
}

pub proof fn lemma_steps_64_is_iter(s: (u64, u64))
    ensures
        steps_64(s) == iter(s, 64),
{
    assert forall|x: (u64, u64)| #[trigger] steps_2(x) == iter(x, 2) by {
        reveal_with_fuel(iter, 3);
    }
    assert forall|x: (u64, u64)| #[trigger] steps_4(x) == iter(x, 4) by {
        lemma_iter_add(x, 2, 2);
    }
    assert forall|x: (u64, u64)| #[trigger] steps_8(x) == iter(x, 8) by {
        lemma_iter_add(x, 4, 4);
    }
    assert forall|x: (u64, u64)| #[trigger] steps_16(x) == iter(x, 16) by {
        lemma_iter_add(x, 8, 8);
    }
    assert forall|x: (u64, u64)| #[trigger] steps_32(x) == iter(x, 32) by {
        lemma_iter_add(x, 16, 16);
    }
    lemma_iter_add(s, 32, 32);
}

/// The unrolled ladder is 128 steps. Only the two halves are used here, so
/// the halves and the iteration stay folded.
pub proof fn lemma_steps_128_is_iter(s: (u64, u64))
    ensures
        steps_128(s) == iter(s, 128),
{
    hide(steps_64);
    hide(iter);
    reveal(steps_128);
    lemma_steps_64_is_iter(s);
    lemma_steps_64_is_iter(steps_64(s));
    lemma_iter_add(s, 64, 64);
}

/// 128 steps end the algorithm on any odd pair of `u64` and leave the gcd.
pub proof fn lemma_steps_128_gcd(s: (u64, u64))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        steps_128(s).1 == 0,
        steps_128(s).0 as nat == gcd(s.0 as nat, s.1 as nat),
{
    lemma_steps_128_is_iter(s);
    lemma_iter_128_gcd(s);
}

/// The same, stated on the iteration so that the unrolled ladder stays out
/// of the proof.
pub proof fn lemma_iter_128_gcd(s: (u64, u64))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        iter(s, 128).1 == 0,
        iter(s, 128).0 as nat == gcd(s.0 as nat, s.1 as nat),
{
    lemma_iter(s, 128);
    let r = iter(s, 128);
    let m = r.0 as nat * r.1 as nat;
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(64, 64);
    assert(m == 0) by (nonlinear_arith)
        requires
            m * pow2(128) <= s.0 as nat * s.1 as nat,
            (s.0 as nat) < pow2(64),
            (s.1 as nat) < pow2(64),
            pow2(128) == pow2(64) * pow2(64),
    ;
    assert(r.1 == 0) by (nonlinear_arith)
        requires
            r.0 as nat * r.1 as nat == 0,
            r.0 != 0,
    ;
    let g = r.0 as nat;
    lemma_mod_self_0(g as int);
    lemma_small_mod(0, g);
    assert(common(g, g, 0));
    assert(common(g, s.0 as nat, s.1 as nat));
    assert forall|d: nat| #[trigger] common(d, s.0 as nat, s.1 as nat) implies divides(d, g) by {
        assert(common(d, g, 0));
    }
    lemma_gcd_unique(s.0 as nat, s.1 as nat, g);
}

// ---- gcd64 --------------------------------------------------------------------

/// The trailing zeros of `left | right` are the fewer of the two.
pub proof fn lemma_u64_or_trailing_zeros(left: u64, right: u64)
    requires
        left != 0,
        right != 0,
    ensures
        (left | right) != 0,
        (left | right).trailing_zeros() == if left.trailing_zeros() <= right.trailing_zeros() {
            left.trailing_zeros()
        } else {
            right.trailing_zeros()
        },
{
    let x = left | right;
    assert(x != 0) by (bit_vector)
        requires
            x == left | right,
            left != 0,
    ;
    vstd::std_specs::bits::axiom_u64_trailing_zeros(left);
    vstd::std_specs::bits::axiom_u64_trailing_zeros(right);
    vstd::std_specs::bits::axiom_u64_trailing_zeros(x);
    let (tl, tr, tx) = (
        left.trailing_zeros() as u64,
        right.trailing_zeros() as u64,
        x.trailing_zeros() as u64,
    );
    let s = if tl <= tr {
        tl
    } else {
        tr
    };
    assert(forall|j: u64|
        j < 64 ==> #[trigger] ((left | right) >> j) & 1 == ((left >> j) & 1) | ((right >> j) & 1))
        by (bit_vector);
    assert(forall|a: u64, b: u64|
        #![trigger a | b]
        (a | b == 0 <==> a == 0 && b == 0)) by (bit_vector);
    assert(forall|a: u64| #![trigger a & 1] a & 1 == 0 || a & 1 == 1) by (bit_vector);
    // Bit `s` of `x` is set, so `tx <= s`; every bit below `s` is clear, so `tx >= s`.
    assert((x >> s) & 1 != 0);
    if tx < s {
        assert((left >> tx) & 1 == 0);
        assert((right >> tx) & 1 == 0);
        assert((x >> tx) & 1 == 1);
        assert(false);
    }
    if tx > s {
        assert((x >> s) & 1 == 0);
        assert(false);
    }
}

/// Two nonzero `u64` have the gcd of their odd parts, scaled by the common
/// power of two.
pub proof fn lemma_odd_parts_gcd_shift(left: u64, right: u64)
    requires
        left != 0,
        right != 0,
    ensures
        odd_state(
            (left >> (left.trailing_zeros() as u64)) as nat,
            (right >> (right.trailing_zeros() as u64)) as nat,
        ),
        (left | right).trailing_zeros() < 64,
        gcd(left as nat, right as nat) == gcd(
            (left >> (left.trailing_zeros() as u64)) as nat,
            (right >> (right.trailing_zeros() as u64)) as nat,
        ) * pow2((left | right).trailing_zeros() as nat),
{
    lemma_u64_odd_part(left);
    lemma_u64_odd_part(right);
    lemma_u64_or_trailing_zeros(left, right);
    lemma_gcd_odd_parts(
        (left >> (left.trailing_zeros() as u64)) as nat,
        (right >> (right.trailing_zeros() as u64)) as nat,
        left.trailing_zeros() as nat,
        right.trailing_zeros() as nat,
    );
}

/// Specification of the production `gcd64`.
#[verifier::opaque]
pub open spec fn gcd64(left: u64, right: u64) -> u64 {
    if left == 0 {
        right
    } else if right == 0 {
        left
    } else {
        steps_128(
            (left >> (left.trailing_zeros() as u64), right >> (right.trailing_zeros() as u64)),
        ).0 << ((left | right).trailing_zeros() as u64)
    }
}

/// The production `gcd64` is Euclid's gcd for every pair of `u64`.
pub proof fn lemma_gcd64(left: u64, right: u64)
    ensures
        gcd64(left, right) as nat == gcd(left as nat, right as nat),
        left != 0 && right != 0 ==> (left | right).trailing_zeros() < 64,
{
    reveal(gcd64);
    if left == 0 {
        lemma_gcd_zero_left(right as nat);
    } else if right == 0 {
    } else {
        lemma_odd_parts_gcd_shift(left, right);
        let pair = (
            left >> (left.trailing_zeros() as u64),
            right >> (right.trailing_zeros() as u64),
        );
        lemma_steps_128_gcd(pair);
        let d = steps_128(pair).0;
        let shift = (left | right).trailing_zeros() as u64;
        lemma_gcd_common(left as nat, right as nat);
        lemma_divides_le(gcd(left as nat, right as nat), left as nat);
        vstd::bits::lemma_u64_shl_is_mul(d, shift);
    }
}

// ---- the 16-step u8 ladder ----------------------------------------------------

/// Specification of the production `step8`.
pub open spec fn step8(s: (u8, u8)) -> (u8, u8) {
    let (left, right) = s;
    if right == 0 {
        (left, right)
    } else {
        let (small, large) = if left > right {
            (right, left)
        } else {
            (left, right)
        };
        let difference = (large - small) as u8;
        if difference == 0 {
            (small, 0u8)
        } else {
            (small, difference >> (difference.trailing_zeros() as u8))
        }
    }
}

/// `k` steps.
pub open spec fn iter8(s: (u8, u8), k: nat) -> (u8, u8)
    decreases k,
{
    if k == 0 {
        s
    } else {
        iter8(step8(s), (k - 1) as nat)
    }
}

/// A nonzero `u8` is its odd part times a power of two.
pub proof fn lemma_u8_odd_part(x: u8)
    requires
        x != 0,
    ensures
        x.trailing_zeros() < 8,
        x as nat == ((x >> (x.trailing_zeros() as u8)) as nat) * pow2(x.trailing_zeros() as nat),
        odd((x >> (x.trailing_zeros() as u8)) as nat),
{
    vstd::std_specs::bits::axiom_u8_trailing_zeros(x);
    let t = x.trailing_zeros() as u8;
    let y = x >> t;
    assert(y % 2 == 1) by (bit_vector)
        requires
            y & 1 == 1,
    ;
    assert((y << t) == x) by (bit_vector)
        requires
            t < 8,
            y == x >> t,
            x << sub(8, t) == 0,
    ;
    vstd::bits::lemma_u8_shr_is_div(x, t);
    let p = pow2(t as nat);
    lemma_pow2_pos(t as nat);
    lemma_fundamental_div_mod(x as int, p as int);
    lemma_mod_bound(x as int, p as int);
    assert(y * p <= x) by (nonlinear_arith)
        requires
            x == p * (x as nat / p) + (x as nat % p),
            y == x as nat / p,
            (x as nat % p) >= 0,
    ;
    vstd::bits::lemma_u8_shl_is_mul(y, t);
}

pub proof fn lemma_step8(s: (u8, u8))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        odd_state(step8(s).0 as nat, step8(s).1 as nat),
        same_common(step8(s).0 as nat, step8(s).1 as nat, s.0 as nat, s.1 as nat),
        s.1 != 0 ==> 2 * (step8(s).0 as nat * step8(s).1 as nat) <= s.0 as nat * s.1 as nat,
        s.1 == 0 ==> step8(s) == s,
{
    let (left, right) = s;
    if right != 0 {
        let (small, large) = if left > right {
            (right, left)
        } else {
            (left, right)
        };
        let difference = (large - small) as u8;
        if difference == 0 {
            assert forall|d: nat|
                #![trigger common(d, small as nat, 0)]
                #![trigger common(d, left as nat, right as nat)]
                common(d, small as nat, 0) <==> common(d, left as nat, right as nat) by {
                if d != 0 {
                    lemma_small_mod(0, d);
                }
            }
            assert(small as nat * 0 == 0) by (nonlinear_arith);
        } else {
            lemma_u8_odd_part(difference);
            let t = difference.trailing_zeros() as nat;
            let new = difference >> (difference.trailing_zeros() as u8);
            lemma_step_math(small as nat, large as nat, new as nat, t);
            assert forall|d: nat|
                #![trigger common(d, small as nat, new as nat)]
                #![trigger common(d, left as nat, right as nat)]
                common(d, small as nat, new as nat) <==> common(d, left as nat, right as nat) by {
                assert(common(d, small as nat, new as nat) <==> common(
                    d,
                    small as nat,
                    large as nat,
                ));
            }
            assert(small as nat * large as nat == left as nat * right as nat) by (nonlinear_arith)
                requires
                    (small == left && large == right) || (small == right && large == left),
            ;
        }
    } else {
        assert forall|d: nat|
            #![trigger common(d, left as nat, right as nat)]
            common(d, left as nat, right as nat) <==> common(d, left as nat, right as nat) by {}
    }
}

pub proof fn lemma_iter8_add(s: (u8, u8), a: nat, b: nat)
    ensures
        iter8(iter8(s, a), b) == iter8(s, a + b),
    decreases a,
{
    if a != 0 {
        lemma_iter8_add(step8(s), (a - 1) as nat, b);
    }
}

pub proof fn lemma_iter8(s: (u8, u8), k: nat)
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        odd_state(iter8(s, k).0 as nat, iter8(s, k).1 as nat),
        same_common(iter8(s, k).0 as nat, iter8(s, k).1 as nat, s.0 as nat, s.1 as nat),
        (iter8(s, k).0 as nat * iter8(s, k).1 as nat) * pow2(k) <= s.0 as nat * s.1 as nat,
    decreases k,
{
    if k == 0 {
        lemma2_to64();
        assert((s.0 as nat * s.1 as nat) * pow2(0) == s.0 as nat * s.1 as nat) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
        assert forall|d: nat|
            #![trigger common(d, s.0 as nat, s.1 as nat)]
            common(d, s.0 as nat, s.1 as nat) <==> common(d, s.0 as nat, s.1 as nat) by {}
    } else {
        let s1 = step8(s);
        lemma_step8(s);
        lemma_iter8(s1, (k - 1) as nat);
        let r = iter8(s, k);
        let m = r.0 as nat * r.1 as nat;
        let m1 = s1.0 as nat * s1.1 as nat;
        let m0 = s.0 as nat * s.1 as nat;
        let h = pow2((k - 1) as nat);
        lemma_pow2_unfold(k);
        assert(m * pow2(k) <= m0) by (nonlinear_arith)
            requires
                m * h <= m1,
                pow2(k) == 2 * h,
                (2 * m1 <= m0) || (m1 == m0 && m0 == 0),
                h >= 0,
        ;
        assert forall|d: nat|
            #![trigger common(d, r.0 as nat, r.1 as nat)]
            #![trigger common(d, s.0 as nat, s.1 as nat)]
            common(d, r.0 as nat, r.1 as nat) <==> common(d, s.0 as nat, s.1 as nat) by {
            assert(common(d, r.0 as nat, r.1 as nat) <==> common(d, s1.0 as nat, s1.1 as nat));
            assert(common(d, s1.0 as nat, s1.1 as nat) <==> common(d, s.0 as nat, s.1 as nat));
        }
    }
}

pub open spec fn steps8_4(s: (u8, u8)) -> (u8, u8) {
    step8(step8(step8(step8(s))))
}

#[verifier::opaque]
pub open spec fn steps8_16(s: (u8, u8)) -> (u8, u8) {
    steps8_4(steps8_4(steps8_4(steps8_4(s))))
}

pub proof fn lemma_steps8_4_is_iter(s: (u8, u8))
    ensures
        steps8_4(s) == iter8(s, 4),
{
    reveal_with_fuel(iter8, 5);
}

/// The unrolled ladder is 16 steps. Only the four quarters are used here, so
/// the quarters and the iteration stay folded.
pub proof fn lemma_steps8_16_is_iter(s: (u8, u8))
    ensures
        steps8_16(s) == iter8(s, 16),
{
    hide(steps8_4);
    hide(iter8);
    reveal(steps8_16);
    let s4 = steps8_4(s);
    let s8 = steps8_4(s4);
    let s12 = steps8_4(s8);
    lemma_steps8_4_is_iter(s);
    lemma_steps8_4_is_iter(s4);
    lemma_steps8_4_is_iter(s8);
    lemma_steps8_4_is_iter(s12);
    lemma_iter8_add(s, 4, 4);
    lemma_iter8_add(s, 8, 4);
    lemma_iter8_add(s, 12, 4);
}

/// 16 steps end the algorithm on any odd pair of `u8` and leave the gcd.
pub proof fn lemma_steps8_16_gcd(s: (u8, u8))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        steps8_16(s).1 == 0,
        steps8_16(s).0 as nat == gcd(s.0 as nat, s.1 as nat),
{
    lemma_steps8_16_is_iter(s);
    lemma_iter8_16_gcd(s);
}

/// The same, stated on the iteration.
pub proof fn lemma_iter8_16_gcd(s: (u8, u8))
    requires
        odd_state(s.0 as nat, s.1 as nat),
    ensures
        iter8(s, 16).1 == 0,
        iter8(s, 16).0 as nat == gcd(s.0 as nat, s.1 as nat),
{
    lemma_iter8(s, 16);
    let r = iter8(s, 16);
    let m = r.0 as nat * r.1 as nat;
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(8, 8);
    assert(m == 0) by (nonlinear_arith)
        requires
            m * pow2(16) <= s.0 as nat * s.1 as nat,
            (s.0 as nat) < pow2(8),
            (s.1 as nat) < pow2(8),
            pow2(16) == pow2(8) * pow2(8),
    ;
    assert(r.1 == 0) by (nonlinear_arith)
        requires
            r.0 as nat * r.1 as nat == 0,
            r.0 != 0,
    ;
    let g = r.0 as nat;
    lemma_mod_self_0(g as int);
    lemma_small_mod(0, g);
    assert(common(g, g, 0));
    assert(common(g, s.0 as nat, s.1 as nat));
    assert forall|d: nat| #[trigger] common(d, s.0 as nat, s.1 as nat) implies divides(d, g) by {
        assert(common(d, g, 0));
    }
    lemma_gcd_unique(s.0 as nat, s.1 as nat, g);
}

// ---- gcd8 ---------------------------------------------------------------------

/// The trailing zeros of `left | right` are the fewer of the two.
pub proof fn lemma_u8_or_trailing_zeros(left: u8, right: u8)
    requires
        left != 0,
        right != 0,
    ensures
        (left | right) != 0,
        (left | right).trailing_zeros() == if left.trailing_zeros() <= right.trailing_zeros() {
            left.trailing_zeros()
        } else {
            right.trailing_zeros()
        },
{
    let x = left | right;
    assert(x != 0) by (bit_vector)
        requires
            x == left | right,
            left != 0,
    ;
    vstd::std_specs::bits::axiom_u8_trailing_zeros(left);
    vstd::std_specs::bits::axiom_u8_trailing_zeros(right);
    vstd::std_specs::bits::axiom_u8_trailing_zeros(x);
    let (tl, tr, tx) = (
        left.trailing_zeros() as u8,
        right.trailing_zeros() as u8,
        x.trailing_zeros() as u8,
    );
    let s = if tl <= tr {
        tl
    } else {
        tr
    };
    assert(forall|j: u8|
        j < 8 ==> #[trigger] ((left | right) >> j) & 1 == ((left >> j) & 1) | ((right >> j) & 1))
        by (bit_vector);
    assert(forall|a: u8, b: u8|
        #![trigger a | b]
        (a | b == 0 <==> a == 0 && b == 0)) by (bit_vector);
    assert(forall|a: u8| #![trigger a & 1] a & 1 == 0 || a & 1 == 1) by (bit_vector);
    // Bit `s` of `x` is set, so `tx <= s`; every bit below `s` is clear, so `tx >= s`.
    assert((x >> s) & 1 != 0);
    if tx < s {
        assert((left >> tx) & 1 == 0);
        assert((right >> tx) & 1 == 0);
        assert((x >> tx) & 1 == 1);
        assert(false);
    }
    if tx > s {
        assert((x >> s) & 1 == 0);
        assert(false);
    }
}

/// Two nonzero `u8` have the gcd of their odd parts, scaled by the common
/// power of two.
pub proof fn lemma_u8_odd_parts_gcd_shift(left: u8, right: u8)
    requires
        left != 0,
        right != 0,
    ensures
        odd_state(
            (left >> (left.trailing_zeros() as u8)) as nat,
            (right >> (right.trailing_zeros() as u8)) as nat,
        ),
        (left | right).trailing_zeros() < 8,
        gcd(left as nat, right as nat) == gcd(
            (left >> (left.trailing_zeros() as u8)) as nat,
            (right >> (right.trailing_zeros() as u8)) as nat,
        ) * pow2((left | right).trailing_zeros() as nat),
{
    lemma_u8_odd_part(left);
    lemma_u8_odd_part(right);
    lemma_u8_or_trailing_zeros(left, right);
    lemma_gcd_odd_parts(
        (left >> (left.trailing_zeros() as u8)) as nat,
        (right >> (right.trailing_zeros() as u8)) as nat,
        left.trailing_zeros() as nat,
        right.trailing_zeros() as nat,
    );
}

/// Specification of the production `gcd8`.
#[verifier::opaque]
pub open spec fn gcd8(left: u8, right: u8) -> u8 {
    if left == 0 {
        right
    } else if right == 0 {
        left
    } else {
        steps8_16(
            (left >> (left.trailing_zeros() as u8), right >> (right.trailing_zeros() as u8)),
        ).0 << ((left | right).trailing_zeros() as u8)
    }
}

/// The production `gcd8` is Euclid's gcd for every pair of `u8`.
pub proof fn lemma_gcd8(left: u8, right: u8)
    ensures
        gcd8(left, right) as nat == gcd(left as nat, right as nat),
        left != 0 && right != 0 ==> (left | right).trailing_zeros() < 8,
{
    reveal(gcd8);
    if left == 0 {
        lemma_gcd_zero_left(right as nat);
    } else if right == 0 {
    } else {
        lemma_u8_odd_parts_gcd_shift(left, right);
        let pair = (
            left >> (left.trailing_zeros() as u8),
            right >> (right.trailing_zeros() as u8),
        );
        lemma_steps8_16_gcd(pair);
        let d = steps8_16(pair).0;
        let shift = (left | right).trailing_zeros() as u8;
        lemma_gcd_common(left as nat, right as nat);
        lemma_divides_le(gcd(left as nat, right as nat), left as nat);
        vstd::bits::lemma_u8_shl_is_mul(d, shift);
    }
}

// ---- the 256-step u128 gcd -----------------------------------------------------

pub proof fn lemma_gcd_sub(a: nat, b: nat)
    requires
        a >= b,
    ensures
        gcd(a, b) == gcd((a - b) as nat, b),
{
    if b != 0 {
        let c = (a - b) as nat;
        assert forall|d: nat|
            #![trigger common(d, a, b)]
            #![trigger common(d, c, b)]
            common(d, a, b) <==> common(d, c, b) by {
            if common(d, a, b) {
                lemma_divides_sub(d, a, b);
            }
            if common(d, c, b) {
                lemma_divides_add(d, c, b);
            }
        }
        lemma_same_common_gcd(a, b, c, b);
    }
}

pub proof fn lemma_gcd_positive(a: nat, b: nat)
    requires
        a != 0 || b != 0,
    ensures
        gcd(a, b) > 0,
        a != 0 ==> gcd(a, b) <= a,
        b != 0 ==> gcd(a, b) <= b,
{
    lemma_gcd_common(a, b);
    if a != 0 {
        lemma_divides_le(gcd(a, b), a);
    }
    if b != 0 {
        lemma_divides_le(gcd(a, b), b);
    }
}

/// Specification of the production `gcd_step`.
pub open spec fn gcd_step(state: (u128, u128, u128)) -> (u128, u128, u128) {
    let (u, v, g) = state;
    let u_even = u & 1 == 0 && u != 0;
    let v_even = v & 1 == 0 && v != 0;
    if u_even && v_even {
        (u >> 1, v >> 1, g << 1)
    } else if u_even && v != 0 {
        (u >> 1, v, g)
    } else if v_even && u != 0 {
        (u, v >> 1, g)
    } else if u & 1 == 1 && v & 1 == 1 {
        if u >= v {
            (((u - v) as u128) >> 1, v, g)
        } else {
            (u, ((v - u) as u128) >> 1, g)
        }
    } else {
        state
    }
}

pub open spec fn gcd_steps_2(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_step(gcd_step(s))
}

pub open spec fn gcd_steps_4(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_2(gcd_steps_2(s))
}

pub open spec fn gcd_steps_8(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_4(gcd_steps_4(s))
}

pub open spec fn gcd_steps_16(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_8(gcd_steps_8(s))
}

pub open spec fn gcd_steps_32(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_16(gcd_steps_16(s))
}

pub open spec fn gcd_steps_64(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_32(gcd_steps_32(s))
}

pub open spec fn gcd_steps_128(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_64(gcd_steps_64(s))
}

#[verifier::opaque]
pub open spec fn gcd_steps_256(s: (u128, u128, u128)) -> (u128, u128, u128) {
    gcd_steps_128(gcd_steps_128(s))
}

/// `k` steps of `gcd_step`.
pub open spec fn giter(s: (u128, u128, u128), k: nat) -> (u128, u128, u128)
    decreases k,
{
    if k == 0 {
        s
    } else {
        giter(gcd_step(s), (k - 1) as nat)
    }
}

/// The state invariant: the factor taken out times the gcd still to find is
/// the gcd `total` of the inputs.
pub open spec fn ginv(s: (u128, u128, u128), total: nat) -> bool {
    s.2 > 0 && s.2 as nat * gcd(s.0 as nat, s.1 as nat) == total
}

pub proof fn lemma_gcd_step(s: (u128, u128, u128), total: nat)
    requires
        ginv(s, total),
        total <= u128::MAX,
    ensures
        ginv(gcd_step(s), total),
        s.0 != 0 && s.1 != 0 ==> 2 * (gcd_step(s).0 as nat * gcd_step(s).1 as nat) <= s.0 as nat
            * s.1 as nat,
        s.0 == 0 || s.1 == 0 ==> gcd_step(s) == s,
{
    let (u, v, g) = s;
    let (un, vn, gn) = (u as nat, v as nat, g as nat);
    assert((u & 1 == 0 <==> u % 2 == 0) && (u & 1 == 1 <==> u % 2 == 1) && u >> 1 == u / 2)
        by (bit_vector);
    assert((v & 1 == 0 <==> v % 2 == 0) && (v & 1 == 1 <==> v % 2 == 1) && v >> 1 == v / 2)
        by (bit_vector);
    lemma2_to64();
    let u_even = u & 1 == 0 && u != 0;
    let v_even = v & 1 == 0 && v != 0;
    if u_even && v_even {
        let (hu, hv) = ((u / 2) as nat, (v / 2) as nat);
        lemma_gcd_scale(2, hu, hv);
        lemma_gcd_positive(hu, hv);
        let h = gcd(hu, hv);
        assert(gn * (2 * h) == (2 * gn) * h) by (nonlinear_arith);
        assert(2 * gn <= (2 * gn) * h) by (nonlinear_arith)
            requires
                h >= 1,
        ;
        vstd::bits::lemma_u128_shl_is_mul(g, 1);
        assert(2 * (hu * hv) <= un * vn) by (nonlinear_arith)
            requires
                un == 2 * hu,
                vn == 2 * hv,
        ;
    } else if u_even && v != 0 {
        let hu = (u / 2) as nat;
        assert(un == hu * pow2(1));
        lemma_gcd_strip_pow2(hu, vn, 1);
        assert(2 * (hu * vn) <= un * vn) by (nonlinear_arith)
            requires
                un == 2 * hu,
        ;
    } else if v_even && u != 0 {
        let hv = (v / 2) as nat;
        assert(vn == hv * pow2(1));
        lemma_gcd_sym(un, vn);
        lemma_gcd_strip_pow2(hv, un, 1);
        lemma_gcd_sym(hv, un);
        assert(2 * (un * hv) <= un * vn) by (nonlinear_arith)
            requires
                vn == 2 * hv,
        ;
    } else if u & 1 == 1 && v & 1 == 1 {
        if u >= v {
            let w = (u - v) as u128;
            assert(w >> 1 == w / 2 && w % 2 == 0) by (bit_vector)
                requires
                    w == u - v,
                    u >= v,
                    u % 2 == 1,
                    v % 2 == 1,
            ;
            let hw = (w / 2) as nat;
            lemma_gcd_sub(un, vn);
            assert(w as nat == hw * pow2(1));
            lemma_gcd_strip_pow2(hw, vn, 1);
            assert(2 * (hw * vn) <= un * vn) by (nonlinear_arith)
                requires
                    2 * hw + vn == un,
            ;
        } else {
            let w = (v - u) as u128;
            assert(w >> 1 == w / 2 && w % 2 == 0) by (bit_vector)
                requires
                    w == v - u,
                    v >= u,
                    u % 2 == 1,
                    v % 2 == 1,
            ;
            let hw = (w / 2) as nat;
            lemma_gcd_sym(un, vn);
            lemma_gcd_sub(vn, un);
            assert(w as nat == hw * pow2(1));
            lemma_gcd_strip_pow2(hw, un, 1);
            lemma_gcd_sym(hw, un);
            assert(2 * (un * hw) <= un * vn) by (nonlinear_arith)
                requires
                    2 * hw + un == vn,
            ;
        }
    } else {
    }
}

pub proof fn lemma_giter_add(s: (u128, u128, u128), a: nat, b: nat)
    ensures
        giter(giter(s, a), b) == giter(s, a + b),
    decreases a,
{
    if a != 0 {
        lemma_giter_add(gcd_step(s), (a - 1) as nat, b);
    }
}

pub proof fn lemma_giter(s: (u128, u128, u128), k: nat, total: nat)
    requires
        ginv(s, total),
        total <= u128::MAX,
    ensures
        ginv(giter(s, k), total),
        (giter(s, k).0 as nat * giter(s, k).1 as nat) * pow2(k) <= s.0 as nat * s.1 as nat,
    decreases k,
{
    if k == 0 {
        lemma2_to64();
        assert((s.0 as nat * s.1 as nat) * pow2(0) == s.0 as nat * s.1 as nat) by (nonlinear_arith)
            requires
                pow2(0) == 1,
        ;
    } else {
        let s1 = gcd_step(s);
        lemma_gcd_step(s, total);
        lemma_giter(s1, (k - 1) as nat, total);
        let r = giter(s, k);
        let m = r.0 as nat * r.1 as nat;
        let m1 = s1.0 as nat * s1.1 as nat;
        let m0 = s.0 as nat * s.1 as nat;
        let h = pow2((k - 1) as nat);
        lemma_pow2_unfold(k);
        if s.0 == 0 || s.1 == 0 {
            assert(m0 == 0) by (nonlinear_arith)
                requires
                    s.0 == 0 || s.1 == 0,
                    m0 == s.0 as nat * s.1 as nat,
            ;
        }
        assert(m * pow2(k) <= m0) by (nonlinear_arith)
            requires
                m * h <= m1,
                pow2(k) == 2 * h,
                (2 * m1 <= m0) || (m1 == m0 && m0 == 0),
                h >= 0,
        ;
    }
}

pub proof fn lemma_gcd_steps_128_is_giter(s: (u128, u128, u128))
    ensures
        gcd_steps_128(s) == giter(s, 128),
{
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_2(x) == giter(x, 2) by {
        reveal_with_fuel(giter, 3);
    }
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_4(x) == giter(x, 4) by {
        lemma_giter_add(x, 2, 2);
    }
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_8(x) == giter(x, 8) by {
        lemma_giter_add(x, 4, 4);
    }
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_16(x) == giter(x, 16) by {
        lemma_giter_add(x, 8, 8);
    }
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_32(x) == giter(x, 32) by {
        lemma_giter_add(x, 16, 16);
    }
    assert forall|x: (u128, u128, u128)| #[trigger] gcd_steps_64(x) == giter(x, 64) by {
        lemma_giter_add(x, 32, 32);
    }
    lemma_giter_add(s, 64, 64);
}

/// The unrolled ladder is 256 steps. Only the two halves are used here, so
/// the halves and the iteration stay folded.
pub proof fn lemma_gcd_steps_256_is_giter(s: (u128, u128, u128))
    ensures
        gcd_steps_256(s) == giter(s, 256),
{
    hide(gcd_steps_128);
    hide(giter);
    reveal(gcd_steps_256);
    lemma_gcd_steps_128_is_giter(s);
    lemma_gcd_steps_128_is_giter(gcd_steps_128(s));
    lemma_giter_add(s, 128, 128);
}

/// 256 steps end the algorithm on any pair of `u128`: one slot is zero and the
/// factor times the other slot is the gcd.
pub proof fn lemma_giter_256(u: u128, v: u128)
    ensures
        ({
            let r = giter((u, v, 1u128), 256);
            (r.0 == 0 || r.1 == 0) && r.2 as nat * (r.0 as nat + r.1 as nat) == gcd(
                u as nat,
                v as nat,
            ) && gcd(u as nat, v as nat) <= u128::MAX
        }),
{
    let total = gcd(u as nat, v as nat);
    if u != 0 || v != 0 {
        lemma_gcd_positive(u as nat, v as nat);
    }
    assert(1 * total == total) by (nonlinear_arith);
    let s = (u, v, 1u128);
    lemma_giter(s, 256, total);
    let r = giter(s, 256);
    let m = r.0 as nat * r.1 as nat;
    lemma2_to64();
    lemma2_to64_rest();
    lemma_pow2_adds(64, 64);
    lemma_pow2_adds(128, 128);
    assert(m == 0) by (nonlinear_arith)
        requires
            m * pow2(256) <= u as nat * v as nat,
            (u as nat) < pow2(128),
            (v as nat) < pow2(128),
            pow2(256) == pow2(128) * pow2(128),
    ;
    assert(r.0 == 0 || r.1 == 0) by (nonlinear_arith)
        requires
            r.0 as nat * r.1 as nat == 0,
    ;
    if r.0 == 0 {
        lemma_gcd_zero_left(r.1 as nat);
    }
}

/// Specification of the production `gcd256`.
#[verifier::opaque]
pub open spec fn gcd256(u: u128, v: u128) -> u128 {
    let r = gcd_steps_256((u, v, 1u128));
    (r.2 * (r.0 + r.1)) as u128
}

/// The production `gcd256` is Euclid's gcd for every pair of `u128`, and its
/// final sum and product do not overflow.
pub proof fn lemma_gcd256(u: u128, v: u128)
    ensures
        ({
            let r = gcd_steps_256((u, v, 1u128));
            r.0 + r.1 <= u128::MAX && r.2 * (r.0 + r.1) <= u128::MAX
        }),
        gcd256(u, v) as nat == gcd(u as nat, v as nat),
{
    reveal(gcd256);
    lemma_gcd_steps_256_is_giter((u, v, 1u128));
    lemma_giter_256(u, v);
}

// ---- coprimality and the canonical-pair rule ----------------------------------

/// Specification of the production `coprime`.
pub open spec fn coprime(left: u64, right: u64) -> bool {
    if (left | right) & 1 == 0 {
        false
    } else {
        steps_128(
            (left >> (left.trailing_zeros() as u64), right >> (right.trailing_zeros() as u64)),
        ).0 == 1
    }
}

/// Specification of the production `is_canonical_pair`.
pub open spec fn is_canonical_pair(numerator: i64, denominator: i64) -> bool {
    if denominator <= 0 {
        false
    } else if numerator == 0 {
        denominator == 1
    } else {
        coprime(numerator.unsigned_abs(), denominator.unsigned_abs())
    }
}

/// The canonical-pair rule in mathematics: a positive denominator, lowest
/// terms by Euclid's gcd, zero only as `0/1`.
pub open spec fn canonical(n: int, d: int) -> bool {
    d > 0 && if n == 0 {
        d == 1
    } else {
        gcd(vstd::math::abs(n) as nat, d as nat) == 1
    }
}

/// `gcd(a, b) == 1` says exactly that no factor above one is shared.
pub proof fn lemma_gcd_one_iff_no_common_factor(a: nat, b: nat)
    requires
        a != 0 || b != 0,
    ensures
        gcd(a, b) == 1 <==> (forall|d: nat| #[trigger] common(d, a, b) ==> d == 1),
{
    lemma_gcd_common(a, b);
    if gcd(a, b) == 1 {
        assert forall|d: nat| #[trigger] common(d, a, b) implies d == 1 by {
            lemma_gcd_greatest(a, b, d);
            lemma_divides_le(d, 1);
        }
    }
}

/// The production coprimality test decides `gcd == 1` for all nonzero `u64`.
pub proof fn lemma_coprime(left: u64, right: u64)
    requires
        left != 0,
        right != 0,
    ensures
        coprime(left, right) == (gcd(left as nat, right as nat) == 1),
{
    let (a, b) = (left as nat, right as nat);
    assert(((left | right) & 1 == 0) <==> (left % 2 == 0 && right % 2 == 0)) by (bit_vector);
    if (left | right) & 1 == 0 {
        assert(common(2, a, b));
        lemma_gcd_greatest(a, b, 2);
        if gcd(a, b) == 1 {
            lemma_divides_le(2, 1);
        }
    } else {
        lemma_odd_parts_gcd(left, right);
        lemma_steps_128_gcd(
            (left >> (left.trailing_zeros() as u64), right >> (right.trailing_zeros() as u64)),
        );
    }
}

/// Two nonzero `u64`, not both even, have the gcd of their odd parts.
pub proof fn lemma_odd_parts_gcd(left: u64, right: u64)
    requires
        left != 0,
        right != 0,
        left % 2 == 1 || right % 2 == 1,
    ensures
        odd_state(
            (left >> (left.trailing_zeros() as u64)) as nat,
            (right >> (right.trailing_zeros() as u64)) as nat,
        ),
        gcd(left as nat, right as nat) == gcd(
            (left >> (left.trailing_zeros() as u64)) as nat,
            (right >> (right.trailing_zeros() as u64)) as nat,
        ),
{
    let (a, b) = (left as nat, right as nat);
    lemma_u64_odd_part(left);
    lemma_u64_odd_part(right);
    let (tl, tr) = (left.trailing_zeros() as nat, right.trailing_zeros() as nat);
    let lo = (left >> (left.trailing_zeros() as u64)) as nat;
    let ro = (right >> (right.trailing_zeros() as u64)) as nat;
    lemma2_to64();
    // The odd one of the two has no power of two to strip.
    if a % 2 == 1 {
        lemma_even_multiple(lo, tl);
        assert(a == lo) by (nonlinear_arith)
            requires
                a == lo * pow2(tl),
                tl == 0,
                pow2(0) == 1,
        ;
        lemma_strip_pow2(lo, ro, tr);
        assert forall|d: nat|
            #![trigger common(d, a, b)]
            #![trigger common(d, lo, ro)]
            common(d, a, b) <==> common(d, lo, ro) by {
            assert(common(d, lo, ro * pow2(tr)) <==> common(d, lo, ro));
        }
    } else {
        lemma_even_multiple(ro, tr);
        assert(b == ro) by (nonlinear_arith)
            requires
                b == ro * pow2(tr),
                tr == 0,
                pow2(0) == 1,
        ;
        lemma_strip_pow2(ro, lo, tl);
        assert forall|d: nat|
            #![trigger common(d, a, b)]
            #![trigger common(d, lo, ro)]
            common(d, a, b) <==> common(d, lo, ro) by {
            assert(common(d, ro, lo * pow2(tl)) <==> common(d, ro, lo));
        }
    }
    lemma_same_common_gcd(a, b, lo, ro);
}

/// `m * 2^t` is even when `t` is positive.
pub proof fn lemma_even_multiple(m: nat, t: nat)
    ensures
        t != 0 ==> (m * pow2(t)) % 2 == 0,
{
    if t != 0 {
        lemma_pow2_unfold(t);
        let h = pow2((t - 1) as nat);
        assert(m * pow2(t) == (m * h) * 2) by (nonlinear_arith)
            requires
                pow2(t) == 2 * h,
        ;
        lemma_mod_multiples_basic((m * h) as int, 2);
    }
}

/// The production canonical-pair test is the canonical-pair rule, for every
/// `i64` pair.
pub proof fn lemma_is_canonical_pair(numerator: i64, denominator: i64)
    ensures
        is_canonical_pair(numerator, denominator) == canonical(numerator as int, denominator as int),
{
    if denominator > 0 && numerator != 0 {
        lemma_coprime(numerator.unsigned_abs(), denominator.unsigned_abs());
    }
}

// ---- reduce -------------------------------------------------------------------

/// Specification of the production `reduce`.
pub open spec fn reduce(numerator: i128, denominator: i128) -> (i128, i128) {
    let (n, d) = (numerator.unsigned_abs(), denominator.unsigned_abs());
    let (n, d): (u128, u128) = if n <= 0xFF && d <= 0xFF {
        let gcd = gcd8(n as u8, d as u8);
        (((n as u8) / gcd) as u128, ((d as u8) / gcd) as u128)
    } else if n <= u64::MAX && d <= u64::MAX {
        let gcd = gcd64(n as u64, d as u64);
        (((n as u64) / gcd) as u128, ((d as u64) / gcd) as u128)
    } else {
        let gcd = gcd256(n, d);
        (n / gcd, d / gcd)
    };
    let signed = n as i128;
    let numerator = if numerator < 0 {
        signed.wrapping_neg()
    } else {
        signed
    };
    (numerator, d as i128)
}

/// The divisors `reduce` divides by are not zero.
pub proof fn lemma_reduce_divisors(numerator: i128, denominator: i128)
    requires
        denominator > 0,
    ensures
        ({
            let (n, d) = (numerator.unsigned_abs(), denominator.unsigned_abs());
            &&& n <= 0xFF && d <= 0xFF ==> gcd8(n as u8, d as u8) != 0
            &&& n <= u64::MAX && d <= u64::MAX ==> gcd64(n as u64, d as u64) != 0
            &&& gcd256(n, d) != 0
        }),
{
    let (n, d) = (numerator.unsigned_abs(), denominator.unsigned_abs());
    lemma_gcd_positive(n as nat, d as nat);
    if n <= 0xFF && d <= 0xFF {
        lemma_gcd8(n as u8, d as u8);
    }
    if n <= u64::MAX && d <= u64::MAX {
        lemma_gcd64(n as u64, d as u64);
    }
    lemma_gcd256(n, d);
}

/// `reduce` returns the fraction in lowest terms: both parts divided exactly
/// by Euclid's gcd of the magnitudes, the sign kept, and the result canonical.
pub proof fn lemma_reduce(numerator: i128, denominator: i128)
    requires
        denominator > 0,
    ensures
        ({
            let g = gcd(vstd::math::abs(numerator as int) as nat, denominator as nat);
            let r = reduce(numerator, denominator);
            &&& g > 0
            &&& r.0 * g == numerator
            &&& r.1 * g == denominator
            &&& canonical(r.0 as int, r.1 as int)
        }),
{
    let (n, d) = (numerator.unsigned_abs(), denominator.unsigned_abs());
    let (nn, dn) = (n as nat, d as nat);
    let g = gcd(nn, dn);
    lemma_gcd_positive(nn, dn);
    lemma_gcd_common(nn, dn);
    lemma_divides_witness(g, nn);
    lemma_divides_witness(g, dn);
    let (qn, qd) = (nn / g, dn / g);
    if n <= 0xFF && d <= 0xFF {
        lemma_gcd8(n as u8, d as u8);
    } else if n <= u64::MAX && d <= u64::MAX {
        lemma_gcd64(n as u64, d as u64);
    } else {
        lemma_gcd256(n, d);
    }
    // The quotients, as the three tiers compute them.
    let r = reduce(numerator, denominator);
    assert(qn <= nn && qd <= dn && qd >= 1) by (nonlinear_arith)
        requires
            nn == g * qn,
            dn == g * qd,
            g >= 1,
            dn >= 1,
    ;
    let signed = (qn as u128) as i128;
    if qn > i128::MAX {
        // Only `i128::MIN` over a gcd of one reaches `2^127`.
        let top = qn as u128;
        assert(top as i128 == i128::MIN) by (bit_vector)
            requires
                top == 0x8000_0000_0000_0000_0000_0000_0000_0000u128,
        ;
        assert(g == 1) by (nonlinear_arith)
            requires
                nn == g * qn,
                nn <= 0x8000_0000_0000_0000_0000_0000_0000_0000,
                qn >= 0x8000_0000_0000_0000_0000_0000_0000_0000,
                g >= 1,
        ;
    }
    assert(r.0 * g == numerator) by (nonlinear_arith)
        requires
            nn == g * qn,
            (numerator >= 0 && r.0 == qn && nn == numerator) || (numerator < 0 && r.0 == -qn && nn
                == -numerator),
    ;
    assert(r.1 * g == denominator) by (nonlinear_arith)
        requires
            dn == g * qd,
            r.1 == qd,
            dn == denominator,
    ;
    if r.0 == 0 {
        assert(nn == 0) by (nonlinear_arith)
            requires
                nn == g * qn,
                qn == 0,
        ;
        lemma_gcd_zero_left(dn);
        assert(qd == 1) by (nonlinear_arith)
            requires
                dn == g * qd,
                g == dn,
                dn >= 1,
        ;
    } else {
        lemma_gcd_scale(g, qn, qd);
        let h = gcd(qn, qd);
        assert(h == 1) by (nonlinear_arith)
            requires
                g * h == g,
                g >= 1,
        ;
    }
}

// ---- canonical fractions are unique --------------------------------------------

/// Euclid's lemma: a divisor of `y z` that is coprime to `y` divides `z`.
pub proof fn lemma_euclid(x: nat, y: nat, z: nat)
    requires
        gcd(x, y) == 1,
        divides(x, y * z),
        z != 0,
    ensures
        divides(x, z),
{
    lemma_gcd_scale(z, x, y);
    assert(z * 1 == z) by (nonlinear_arith);
    lemma_divides_intro(x, z);
    lemma_mul_is_commutative(x as int, z as int);
    lemma_mul_is_commutative(y as int, z as int);
    lemma_gcd_greatest(z * x, z * y, x);
}

/// Two canonical fractions are the same pair exactly when they cross-multiply
/// equal.
pub proof fn lemma_canonical_unique(a: int, p: int, c: int, q: int)
    requires
        canonical(a, p),
        canonical(c, q),
    ensures
        (a == c && p == q) <==> a * q == c * p,
{
    if a * q == c * p {
        if a == 0 {
            assert(c == 0) by (nonlinear_arith)
                requires
                    c * p == 0,
                    p > 0,
            ;
        } else {
            assert(c != 0) by (nonlinear_arith)
                requires
                    a * q == c * p,
                    a != 0,
                    q > 0,
            ;
            let (an, cn) = (vstd::math::abs(a) as nat, vstd::math::abs(c) as nat);
            let (pn, qn) = (p as nat, q as nat);
            assert(an * qn == cn * pn) by (nonlinear_arith)
                requires
                    a * q == c * p,
                    p > 0,
                    q > 0,
                    an == if a < 0 { -a } else { a },
                    cn == if c < 0 { -c } else { c },
                    pn == p,
                    qn == q,
            ;
            // p divides |a| q and is coprime to |a|, so p divides q; and q divides p.
            lemma_divides_intro(pn, cn);
            lemma_mul_is_commutative(cn as int, pn as int);
            lemma_gcd_sym(pn, an);
            lemma_euclid(pn, an, qn);
            lemma_divides_intro(qn, an);
            lemma_mul_is_commutative(an as int, qn as int);
            lemma_gcd_sym(qn, cn);
            lemma_euclid(qn, cn, pn);
            lemma_divides_antisym(pn, qn);
            assert(a == c) by (nonlinear_arith)
                requires
                    a * q == c * p,
                    p == q,
                    q > 0,
            ;
        }
    }
}

} // verus!
