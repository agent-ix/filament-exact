#!/usr/bin/env python3
"""Mutants of the native float arithmetic the Verus proofs cover (FR-405
Behavior 23, TC-953).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports a failed proof obligation.
Every mutant but the two of the square-root loop also edits the specification
function of the same name, so the production function still equals its
specification and only a theorem can fail. The two loop mutants have no
specification copy: the loop invariant is the proof they fail. The runner is the one of `scripts/ir1544_verus_mutants.py`.

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
    ("nearest-away-rounds-ties-down-code-and-spec", [
        (FLOAT, "        RoundingMode::NearestAway => !below_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,",
         "        RoundingMode::NearestAway => above_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,"),
        (PROOFS, "        RoundingMode::NearestAway => !below_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,",
         "        RoundingMode::NearestAway => above_half,\n        RoundingMode::TowardZero => false,\n        RoundingMode::TowardPositive => inexact && !negative,")]),
    ("toward-positive-ignores-the-sign-code-and-spec", [
        (FLOAT, "        RoundingMode::TowardPositive => inexact && !negative,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (if up",
         "        RoundingMode::TowardPositive => inexact,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (if up"),
        (PROOFS, "        RoundingMode::TowardPositive => inexact && !negative,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (\n        if up {",
         "        RoundingMode::TowardPositive => inexact,\n        RoundingMode::TowardNegative => inexact && negative,\n    };\n    (\n        if up {")]),
    ("carry-not-renormalised-code-and-spec", [
        (FLOAT, "        kept >>= 1;\n        index += 1;", "        index += 1;"),
        (PROOFS, "                (kept0 >> 1u32, (index0 + 1) as u64)", "                (kept0, (index0 + 1) as u64)")]),
    ("underflow-without-inexact-code-and-spec", [
        (FLOAT, "    if tiny && inexact {", "    if tiny {"),
        (PROOFS, "                    if tiny && inexact {", "                    if tiny {")]),
    ("overflow-toward-zero-goes-to-infinity-code-and-spec", [
        (FLOAT, "            RoundingMode::TowardZero => false,\n            RoundingMode::TowardPositive => !negative,",
         "            RoundingMode::TowardZero => true,\n            RoundingMode::TowardPositive => !negative,"),
        (PROOFS, "                    RoundingMode::TowardZero => false,\n                    RoundingMode::TowardPositive => !negative,",
         "                    RoundingMode::TowardZero => true,\n                    RoundingMode::TowardPositive => !negative,")]),
    ("divide-one-guard-bit-fewer-code-and-spec", [
        (FLOAT, "            let numerator = u128::from(dividend) << (fmt.fraction_bits + 3);",
         "            let numerator = u128::from(dividend) << (fmt.fraction_bits + 2);"),
        (PROOFS, "            let numerator = (dividend as u128) << ((fmt.fraction_bits + 3) as u32);",
         "            let numerator = (dividend as u128) << ((fmt.fraction_bits + 2) as u32);")]),
    ("divide-sticky-dropped-code-and-spec", [
        (FLOAT, "                sticky: !numerator.is_multiple_of(divisor),", "                sticky: false,"),
        (PROOFS, "                    sticky: !(numerator % divisor == 0),", "                    sticky: false,")]),
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
    ("product-exponents-subtracted-code-and-spec", [
        (FLOAT, "            exp: ex + ey,", "            exp: ex - ey,"),
        (PROOFS, "exp: (ex + ey) as i32", "exp: (ex - ey) as i32")]),
    ("justify-one-bit-short-code-and-spec", [
        (FLOAT, "    let shift = significand.leading_zeros() - (63 - fmt.fraction_bits);",
         "    let shift = significand.leading_zeros() - (64 - fmt.fraction_bits);"),
        (PROOFS, "- (63 - fmt.fraction_bits)) as u32;", "- (64 - fmt.fraction_bits)) as u32;")]),
    ("term-sign-flipped-code-and-spec", [
        (FLOAT, "            negative,\n            mag: u128::from(significand),",
         "            negative: !negative,\n            mag: u128::from(significand),"),
        (PROOFS, "            negative,\n            mag: significand as u128,",
         "            negative: !negative,\n            mag: significand as u128,")]),
    ("normalised-one-bit-short-code-and-spec", [
        (FLOAT, "    let shift = term.mag.leading_zeros() - (127 - WIDE_TOP);",
         "    let shift = term.mag.leading_zeros() - (128 - WIDE_TOP);"),
        (PROOFS, "pub(super) open spec fn normalised(term: Term) -> Term {\n    let shift = (term.mag.leading_zeros() - 2) as u32;",
         "pub(super) open spec fn normalised(term: Term) -> Term {\n    let shift = (term.mag.leading_zeros() - 3) as u32;")]),
    ("least-normal-exponent-off-by-one-code-and-spec", [
        (FLOAT, "        1 - self.bias()\n    }", "        2 - self.bias()\n    }"),
        (PROOFS, "    (1 - bias(fmt)) as i32", "    (2 - bias(fmt)) as i32")]),
    ("fraction-one-too-many-code-and-spec", [
        (FLOAT, "        self.fraction_bits as i32\n", "        self.fraction_bits as i32 + 1\n"),
        (PROOFS, "    fmt.fraction_bits as i32\n}", "    (fmt.fraction_bits + 1) as i32\n}")]),
    ("bias-one-too-many-code-and-spec", [
        (FLOAT, "        (1 << (self.exponent_bits - 1)) - 1\n", "        1 << (self.exponent_bits - 1)\n"),
        (PROOFS, "        127\n    } else {\n        1023\n", "        128\n    } else {\n        1024\n")]),
    ("quantum-min-off-by-one-code-and-spec", [
        (FLOAT, "        1 - self.bias() - self.fraction()\n", "        2 - self.bias() - self.fraction()\n"),
        (PROOFS, "    (1 - bias(fmt) - fraction(fmt)) as i32", "    (2 - bias(fmt) - fraction(fmt)) as i32")]),
    ("sign-mask-one-bit-low-code-and-spec", [
        (FLOAT, "        1 << (self.exponent_bits + self.fraction_bits)\n",
         "        1 << (self.exponent_bits + self.fraction_bits - 1)\n"),
        (PROOFS, "        0x8000_0000\n    } else {\n        0x8000_0000_0000_0000\n",
         "        0x4000_0000\n    } else {\n        0x4000_0000_0000_0000\n")]),
    ("exponent-ones-one-too-many-code-and-spec", [
        (FLOAT, "        (1 << self.exponent_bits) - 1\n", "        1 << self.exponent_bits\n"),
        (PROOFS, "        0xFF\n    } else {\n        0x7FF\n", "        0x100\n    } else {\n        0x800\n")]),
    ("exponent-mask-one-bit-high-code-and-spec", [
        (FLOAT, "        self.exponent_ones() << self.fraction_bits\n",
         "        self.exponent_ones() << (self.fraction_bits + 1)\n"),
        (PROOFS, "        0x7F80_0000\n    } else {\n        0x7FF0_0000_0000_0000\n",
         "        0xFF00_0000\n    } else {\n        0xFFE0_0000_0000_0000\n")]),
    ("fraction-mask-one-too-many-code-and-spec", [
        (FLOAT, "        (1 << self.fraction_bits) - 1\n", "        1 << self.fraction_bits\n"),
        (PROOFS, "        0x7F_FFFF\n    } else {\n        0xF_FFFF_FFFF_FFFF\n",
         "        0x80_0000\n    } else {\n        0x10_0000_0000_0000\n")]),
    ("quiet-bit-one-bit-low-code-and-spec", [
        (FLOAT, "        1 << (self.fraction_bits - 1)\n", "        1 << (self.fraction_bits - 2)\n"),
        (PROOFS, "        0x40_0000\n    } else {\n        0x8_0000_0000_0000\n",
         "        0x20_0000\n    } else {\n        0x4_0000_0000_0000\n")]),
    ("sign-inverted-code-and-spec", [
        (FLOAT, "        if negative {\n            self.sign_mask()", "        if !negative {\n            self.sign_mask()"),
        (PROOFS, "    if negative {\n        sign_mask(fmt)", "    if !negative {\n        sign_mask(fmt)")]),
    ("infinity-loses-its-sign-code-and-spec", [
        (FLOAT, "        self.sign(negative) | self.exponent_mask()\n", "        self.exponent_mask()\n"),
        (PROOFS, "    (sign(fmt, negative) + exponent_mask(fmt)) as u64", "    exponent_mask(fmt)")]),
    ("max-finite-fraction-dropped-code-and-spec", [
        (FLOAT, "            | (self.exponent_mask() - (1 << self.fraction_bits))\n            | self.fraction_mask()\n",
         "            | (self.exponent_mask() - (1 << self.fraction_bits))\n"),
        (PROOFS, "        0x7F7F_FFFFu64\n    } else {\n        0x7FEF_FFFF_FFFF_FFFFu64\n",
         "        0x7F00_0000u64\n    } else {\n        0x7FE0_0000_0000_0000u64\n")]),
    ("canonical-nan-is-signaling-code-and-spec", [
        (FLOAT, "        self.exponent_mask() | self.quiet_bit()\n",
         "        self.exponent_mask() | (self.quiet_bit() >> 1)\n"),
        (PROOFS, "        0x7FC0_0000\n    } else {\n        0x7FF8_0000_0000_0000\n",
         "        0x7FA0_0000\n    } else {\n        0x7FF4_0000_0000_0000\n")]),
    ("zero-bits-sign-inverted-code-and-spec", [
        (FLOAT, "        self.sign(negative)\n    }\n}", "        self.sign(!negative)\n    }\n}"),
        (PROOFS, "-> u64 {\n    sign(fmt, negative)\n}", "-> u64 {\n    sign(fmt, !negative)\n}")]),
    ("decode-drops-the-implicit-bit-code-and-spec", [
        (FLOAT, "            significand: fraction | (1 << fmt.fraction_bits),", "            significand: fraction,"),
        (PROOFS, "            significand: fraction_field | ((fraction_mask(fmt) + 1) as u64),",
         "            significand: fraction_field,")]),
    ("multiply-claims-a-lost-remainder-code-and-spec", [
        (FLOAT, "                    exp: product.exp,\n                    sticky: false,",
         "                    exp: product.exp,\n                    sticky: true,"),
        (PROOFS, "Exact::Real { negative, mag: product.mag, exp: product.exp, sticky: false }",
         "Exact::Real { negative, mag: product.mag, exp: product.exp, sticky: true }")]),
    ("fma-squares-the-first-factor-code-and-spec", [
        (FLOAT, "        (false, _) => Step::Finite(sum(rounding, product_term(a, b), term(c))),",
         "        (false, _) => Step::Finite(sum(rounding, product_term(a, a), term(c))),"),
        (PROOFS, "                    Step::Finite(sum(rounding, product_term(a, b), term(c)))",
         "                    Step::Finite(sum(rounding, product_term(a, a), term(c)))")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
