#!/usr/bin/env python3
"""Mutants of the native float arithmetic the Verus proofs cover (FR-405
Behavior 22, TC-953).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports a failed proof obligation.
A mutant that also edits the specification function of the same name leaves
the production function equal to its specification, so only a theorem can
fail. The runner is the one of `scripts/ir1544_verus_mutants.py`.

    VERUS=/path/to/verus python3 scripts/ir1551_verus_mutants.py [mutant-id ...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ir1544_verus_mutants import run  # noqa: E402

FLOAT = "native_float.rs"
PROOFS = "native_float/verus_proofs.rs"

# (id, [(file, old, new), ...])
MUTANTS = [
    ("ties-to-odd-code-and-spec", [
        (FLOAT, "above_half || (tie && kept & 1 == 1),", "above_half || (tie && kept & 1 == 0),"),
        (PROOFS, "above_half || (tie && kept & 1 == 1),", "above_half || (tie && kept & 1 == 0),")]),
    ("nearest-away-rounds-ties-down", [
        (FLOAT, "        RoundingMode::NearestAway => !below_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,",
         "        RoundingMode::NearestAway => above_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,")]),
    ("toward-positive-ignores-the-sign", [
        (FLOAT, "        RoundingMode::TowardPositive => inexact && !negative,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (if up",
         "        RoundingMode::TowardPositive => inexact,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (if up")]),
    ("carry-not-renormalised", [
        (FLOAT, "        kept >>= 1;\n        index += 1;", "        index += 1;")]),
    ("underflow-without-inexact-code-and-spec", [
        (FLOAT, "    if tiny && inexact {", "    if tiny {"),
        (PROOFS, "                    if tiny && inexact {", "                    if tiny {")]),
    ("overflow-toward-zero-goes-to-infinity", [
        (FLOAT, "            RoundingMode::TowardZero => false,\n            RoundingMode::TowardPositive => !negative,",
         "            RoundingMode::TowardZero => true,\n            RoundingMode::TowardPositive => !negative,")]),
    ("divide-one-guard-bit-fewer-code-and-spec", [
        (FLOAT, "            let numerator = u128::from(dividend) << (fmt.fraction_bits + 3);",
         "            let numerator = u128::from(dividend) << (fmt.fraction_bits + 2);"),
        (PROOFS, "            let numerator = (dividend as u128) << ((fmt.fraction_bits + 3) as u32);",
         "            let numerator = (dividend as u128) << ((fmt.fraction_bits + 2) as u32);")]),
    ("divide-sticky-dropped", [
        (FLOAT, "                sticky: !numerator.is_multiple_of(divisor),", "                sticky: false,")]),
    ("divide-by-zero-raises-invalid", [
        (FLOAT, "            IeeeFlags::EMPTY.with(IeeeFlag::DivideByZero),",
         "            IeeeFlags::EMPTY.with(IeeeFlag::Invalid),")]),
    ("sqrt-odd-exponent-not-doubled-code-and-spec", [
        (FLOAT, "                significand <<= 1;\n", ""),
        (PROOFS, "        (s << 1u32, (e - 1) as i32)", "        (s, (e - 1) as i32)")]),
    ("sqrt-digit-accepted-only-below", [
        (FLOAT, "        if trial * trial <= radicand {", "        if trial * trial < radicand {")]),
    ("sqrt-one-step-fewer", [
        (FLOAT, "const SQRT_STEPS: u32 = 56;", "const SQRT_STEPS: u32 = 55;")]),
    ("sum-borrow-for-the-sticky-dropped-code-and-spec", [
        (FLOAT, "        (high.negative, high.mag - shifted - 1, true)",
         "        (high.negative, high.mag - shifted, true)"),
        (PROOFS, "            (high.negative, (high.mag - shifted - 1) as u128, true)",
         "            (high.negative, (high.mag - shifted) as u128, true)")]),
    ("fma-infinite-product-takes-the-addend-sign", [
        (FLOAT, "        (true, _) => decided(fmt.infinity(product_negative)),",
         "        (true, _) => decided(fmt.infinity(c.negative())),")]),
    ("product-exponents-subtracted", [
        (FLOAT, "            exp: ex + ey,", "            exp: ex - ey,")]),
    ("signaling-nan-read-as-quiet", [
        (FLOAT, "                signaling: fraction & fmt.quiet_bit() == 0,",
         "                signaling: fraction & fmt.quiet_bit() != 0,")]),
    ("justify-one-bit-short", [
        (FLOAT, "    let shift = significand.leading_zeros() - (63 - fmt.fraction_bits);",
         "    let shift = significand.leading_zeros() - (64 - fmt.fraction_bits);")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
