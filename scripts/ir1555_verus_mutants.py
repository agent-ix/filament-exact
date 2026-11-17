#!/usr/bin/env python3
"""Mutants of the native Decimal operations and ordering the Verus proofs
cover (FR-403 Behavior 41, TC-951).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports a failed proof obligation.
Most production functions here carry a mathematical postcondition that shares
no text with their body, so an edit of the body alone fails that statement.
A function whose specification is a copy of its body (`capacity`, `lift_fits`,
`tie`, the strips and `loss_record`) is edited together with that copy, so the
function still equals its specification and only a theorem can fail. The
runner is the one of `scripts/ir1544_verus_mutants.py`.

    VERUS=/path/to/verus python3 scripts/ir1555_verus_mutants.py [mutant-id ...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ir1544_verus_mutants import run  # noqa: E402

OPS = "native_decimal/operations.rs"
PROOFS = "native_decimal/operations/verus_proofs.rs"

# (id, [(file, old, new), ...])
MUTANTS = [
    ("capacity-names-the-exponent-code-and-spec", [
        (OPS, "    found.ok_or(NativeDecimalCapacityRefusal { quantity })",
         "    found.ok_or(NativeDecimalCapacityRefusal {\n        quantity: NativeDecimalCapacityQuantity::Exponent,\n    })"),
        (PROOFS, "        None => Err(NativeDecimalCapacityRefusal { quantity }),",
         "        None => Err(NativeDecimalCapacityRefusal { quantity: NativeDecimalCapacityQuantity::Exponent }),")]),
    ("lift-of-39-admitted-code-and-spec", [
        (OPS, "    target - working_scale.min(target) <= u64::from(MAX_SHIFT)",
         "    target - working_scale.min(target) <= u64::from(MAX_SHIFT) + 1"),
        (PROOFS, "    target - (if working < target { working } else { target as u64 }) <= 38",
         "    target - (if working < target { working } else { target as u64 }) <= 39")]),
    ("bits-of-zero-is-two", [
        (OPS, "    u64::from((128 - magnitude.leading_zeros()).max(1))", "    u64::from((128 - magnitude.leading_zeros()).max(2))")]),
    ("digits-estimate-doubled", [
        (OPS, "    let estimate = (bits * 1233) >> 12;", "    let estimate = (bits * 1233) >> 11;")]),
    ("digits-comparison-inverted", [
        (OPS, "    (estimate as u64 + 1) - u64::from(magnitude < power)", "    (estimate as u64 + 1) - u64::from(magnitude >= power)")]),
    ("shift-adds-the-power", [
        (OPS, "    c.checked_mul(POWERS_OF_TEN[shift as usize])", "    c.checked_add(POWERS_OF_TEN[shift as usize])")]),
    ("combine-right-shift-unchecked", [
        (OPS, "(ka <= MAX_SHIFT && kb <= MAX_SHIFT && lift_fits(ty, u64::from(scale))).then_some(()),",
         "(ka <= MAX_SHIFT && lift_fits(ty, u64::from(scale))).then_some(()),")]),
    ("combine-difference-is-a-sum", [
        (OPS, "            xa.checked_sub(xb),", "            xa.checked_add(xb),")]),
    ("combine-sum-named-difference", [
        (OPS, "        capacity(xa.checked_add(xb), NativeDecimalCapacityQuantity::Sum)?",
         "        capacity(xa.checked_add(xb), NativeDecimalCapacityQuantity::Difference)?")]),
    ("multiply-scale-is-the-larger", [
        (OPS, "    let scale = u64::from(a.scale) + u64::from(b.scale);",
         "    let scale = u64::from(a.scale).max(u64::from(b.scale));")]),
    ("negate-keeps-the-sign", [
        (OPS, "            a.coefficient.checked_neg(),", "            Some(a.coefficient),")]),
    ("divide-shifts-swapped", [
        (OPS, "        up.saturating_sub(u64::from(a.scale)),\n        u64::from(a.scale).saturating_sub(up),",
         "        u64::from(a.scale).saturating_sub(up),\n        up.saturating_sub(u64::from(a.scale)),")]),
    ("divide-zero-divisor-planned", [
        (OPS, "    if b.coefficient == 0 {\n        // The owner returns", "    if false {\n        // The owner returns")]),
    ("quotient-sign-from-the-numerator", [
        (OPS, "    let negative = (n < 0) != (d < 0);", "    let negative = n < 0;")]),
    ("quotient-least-value-refused", [
        (OPS, "        if magnitude <= 1_u128 << 127 {", "        if magnitude < 1_u128 << 127 {")]),
    ("quotient-not-reduced", [
        (OPS, "    let (magnitude, denominator) = (n.unsigned_abs() / gcd, d.unsigned_abs() / gcd);",
         "    let (magnitude, denominator) = (n.unsigned_abs(), d.unsigned_abs());")]),
    ("toward-zero-away-from-zero", [
        (OPS, "        RoundingMode::TowardZero => Ok(if negative { ceiling } else { floor }),",
         "        RoundingMode::TowardZero => Ok(if negative { floor } else { ceiling }),")]),
    ("toward-positive-floors", [
        (OPS, "        RoundingMode::TowardPositive => Ok(ceiling),", "        RoundingMode::TowardPositive => Ok(floor),")]),
    ("nearest-below-half-rounds-up", [
        (OPS, "                Ordering::Less => floor,", "                Ordering::Less => ceiling,")]),
    ("doubled-remainder-wraps", [
        (OPS, "                remainder.checked_add(remainder),", "                Some(remainder.wrapping_add(remainder)),")]),
    ("tie-to-odd-code-and-spec", [
        (OPS, "    let even = if floor & 1 == 0 { floor } else { ceiling };",
         "    let even = if floor & 1 == 0 { ceiling } else { floor };"),
        (PROOFS, "        if floor % 2 == 0 {\n            floor\n        } else {\n            ceiling\n        }",
         "        if floor % 2 == 0 {\n            ceiling\n        } else {\n            floor\n        }")]),
    ("tie-away-toward-zero-code-and-spec", [
        (OPS, "    let away = if negative { floor } else { ceiling };",
         "    let away = if negative { ceiling } else { floor };"),
        (PROOFS, "    } else if negative {\n        floor\n    } else {\n        ceiling\n    }\n}",
         "    } else if negative {\n        ceiling\n    } else {\n        floor\n    }\n}")]),
    ("strip-twos-ignores-the-limit-code-and-spec", [
        (OPS, "    let count = u64::from(magnitude.trailing_zeros()).min(limit);",
         "    let count = u64::from(magnitude.trailing_zeros());"),
        (PROOFS, "    let count = if zeros <= limit {\n        zeros\n    } else {\n        limit\n    };",
         "    let count = zeros;")]),
    ("strip-five-ignores-the-limit-code-and-spec", [
        (OPS, "    if width <= limit - count && magnitude.is_multiple_of(power) {",
         "    if magnitude.is_multiple_of(power) {"),
        (PROOFS, "    if width <= limit - count && magnitude % power == 0 {", "    if magnitude % power == 0 {")]),
    ("strip-fives-rung-missing-code-and-spec", [
        (OPS, "    let (m, c) = strip_five(m, c, limit, 4, FIVE_LADDER[2]);\n", ""),
        (PROOFS, "    let (m, c) = strip_five(m, c, limit, 4, 625);\n", "")]),
    ("five-ladder-entry-wrong", [
        (OPS, "    390_625,\n", "    390_626,\n")]),
    ("loss-twos-not-cancelled-code-and-spec", [
        (OPS, "        twos: denominator_twos + (ten_exponent - cancelled_twos),",
         "        twos: denominator_twos + ten_exponent,"),
        (PROOFS, "        twos: (denominator_twos + (ten_exponent - cancelled_twos)) as u64,",
         "        twos: (denominator_twos + ten_exponent) as u64,")]),
    ("loss-fives-take-the-twos-code-and-spec", [
        (OPS, "        fives: denominator_fives + (ten_exponent - cancelled_fives),",
         "        fives: denominator_fives + (ten_exponent - cancelled_twos),"),
        (PROOFS, "        fives: (denominator_fives + (ten_exponent - cancelled_fives)) as u64,",
         "        fives: (denominator_fives + (ten_exponent - cancelled_twos)) as u64,")]),
    ("loss-numerator-loses-its-sign-code-and-spec", [
        (OPS, "        numerator: if numerator < 0 {\n            signed.wrapping_neg()\n        } else {\n            signed\n        },\n        cofactor: cofactor as i128,",
         "        numerator: signed,\n        cofactor: cofactor as i128,"),
        (PROOFS, "        numerator: if numerator < 0 {\n            signed.wrapping_neg()\n        } else {\n            signed\n        },\n        cofactor: cofactor as i128,",
         "        numerator: signed,\n        cofactor: cofactor as i128,")]),
    ("exact-tail-lift-not-applied", [
        (OPS, "        shifted(coefficient, lift),\n        NativeDecimalCapacityQuantity::RetainedCoefficient,",
         "        Some(coefficient),\n        NativeDecimalCapacityQuantity::RetainedCoefficient,")]),
    ("rounding-tail-rounds-under-exact", [
        (OPS, "    if ty.rounding == RoundingMode::Exact {\n        return Ok(Tail::Inexact);\n    }\n", "")]),
    ("rounding-tail-retains-zero", [
        (OPS, "        retained: rounded,", "        retained: 0,")]),
    ("proxy-taken-at-equal-digits", [
        (OPS, "    if excess > digits_of(value.unsigned_abs()) {", "    if excess >= digits_of(value.unsigned_abs()) {")]),
    ("proxy-of-a-fifth", [
        (OPS, "        return rounding_tail(ty, (value.signum(), 10), (value, 1, working));",
         "        return rounding_tail(ty, (value.signum(), 5), (value, 1, working));")]),
    ("scaled-zero-not-placed-exactly", [
        (OPS, "    if value == 0 {\n        return exact_tail(ty, 0, ty.max_scale);\n    }\n", "")]),
    ("scaled-power-of-38-refused", [
        (OPS, "        (excess <= u64::from(MAX_SHIFT)).then_some(()),", "        (excess < u64::from(MAX_SHIFT)).then_some(()),")]),
    ("scaled-power-ground-named-exponent", [
        (OPS, "        (excess <= u64::from(MAX_SHIFT)).then_some(()),\n        NativeDecimalCapacityQuantity::PowerOfTen,",
         "        (excess <= u64::from(MAX_SHIFT)).then_some(()),\n        NativeDecimalCapacityQuantity::Exponent,")]),
    ("scaled-units-not-reduced", [
        (OPS, "    rounding_tail(ty, (value / gcd, power / gcd), (value, 1, working))",
         "    rounding_tail(ty, (value, power), (value, 1, working))")]),
    ("scaled-loss-at-the-excess", [
        (OPS, "    rounding_tail(ty, (value / gcd, power / gcd), (value, 1, working))",
         "    rounding_tail(ty, (value / gcd, power / gcd), (value, 1, excess))")]),
    ("quotient-loss-without-the-target-scale", [
        (OPS, "        (numerator, denominator, u64::from(ty.max_scale)),", "        (numerator, denominator, 0),")]),
    ("quotient-integer-test-wrong", [
        (OPS, "    if denominator == 1 {\n        return exact_tail(ty, numerator, ty.max_scale);",
         "    if denominator == 2 {\n        return exact_tail(ty, numerator, ty.max_scale);")]),
    ("plan-round-negates", [
        (OPS, "        NativeDecimalOperation::Round(a) => plan_unary(ty, a, false),",
         "        NativeDecimalOperation::Round(a) => plan_unary(ty, a, true),")]),
    ("plan-subtract-adds", [
        (OPS, "        NativeDecimalOperation::Subtract(a, b) => plan_combine(ty, a, b, true),",
         "        NativeDecimalOperation::Subtract(a, b) => plan_combine(ty, a, b, false),")]),
    ("admit-zero-divisor-as-inexact", [
        (OPS, "            None => Tail::DivisionByZero,", "            None => Tail::Inexact,")]),
    ("settle-membership-inverted", [
        (OPS, "            if ty.admits(&placed) {", "            if !ty.admits(&placed) {")]),
    ("settle-retains-the-placed-coefficient", [
        (OPS, "                        value: NativeDecimal::from_retained(placement.retained, ty.max_scale),",
         "                        value: NativeDecimal::from_retained(placement.coefficient, ty.max_scale),")]),
    ("settle-rounding-flag-inverted", [
        (OPS, "                    rounding: placement.loss.is_some(),", "                    rounding: placement.loss.is_none(),")]),
    ("widening-carry-dropped", [
        (OPS, "    let high = hh + (lh >> 64) + (hl >> 64) + (middle >> 64);",
         "    let high = hh + (lh >> 64) + (hl >> 64);")]),
    ("order-digit-counts-reversed", [
        (OPS, "        return shifted_digits.cmp(&digits);", "        return digits.cmp(&shifted_digits);")]),
    ("order-high-word-ignored", [
        (OPS, "    (high, low).cmp(&(0, y))", "    low.cmp(&y)")]),
    ("order-larger-scale-not-reversed", [
        (OPS, "        compare_shifted_magnitude(y, left.scale - right.scale, x).reverse()",
         "        compare_shifted_magnitude(y, left.scale - right.scale, x)")]),
    ("order-negatives-not-reversed", [
        (OPS, "    if left_sign < 0 {\n        magnitude.reverse()\n    } else {\n        magnitude\n    }",
         "    magnitude")]),
    ("order-by-sign-reversed", [
        (OPS, "        return left_sign.cmp(&right_sign);", "        return right_sign.cmp(&left_sign);")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
