#!/usr/bin/env python3
"""Mutants of the native Decimal constructor and normalized view the Verus
proofs cover (FR-402 Behavior 11, TC-950).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports a failed proof obligation.
A function whose specification is a copy of its body is edited together with
that copy, so the function still equals its specification and only a theorem
can fail. The constructor's postcondition and the `normalized` wrapper's are
statements, not copies, and those two mutants edit the code alone. The runner
is the one of `scripts/ir1544_verus_mutants.py`.

    VERUS=/path/to/verus python3 scripts/ir1554_verus_mutants.py [mutant-id ...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ir1544_verus_mutants import run  # noqa: E402

DECIMAL = "native_decimal.rs"
PROOFS = "native_decimal/verus_proofs.rs"

# (id, [(file, old, new), ...])
MUTANTS = [
    ("strip-ignores-the-scale-code-and-spec", [
        (DECIMAL, "    if scale >= places && coefficient % power == 0 {", "    if coefficient % power == 0 {"),
        (PROOFS, "    if scale >= places && rust_rem(coefficient as int, power as int) == 0 {",
         "    if rust_rem(coefficient as int, power as int) == 0 {")]),
    ("strip-keeps-the-scale-code-and-spec", [
        (DECIMAL, "        (coefficient / power, scale - places)", "        (coefficient / power, scale)"),
        (PROOFS, "        (rust_div(coefficient as int, power as int) as i128, (scale - places) as u32)",
         "        (rust_div(coefficient as int, power as int) as i128, scale)")]),
    ("largest-stage-missing-code-and-spec", [
        (DECIMAL, "        let (coefficient, scale) = strip_zeros(coefficient, scale, 32, POWERS_OF_TEN[32]);\n", ""),
        (PROOFS, "        let (c, s) = strip_zeros(coefficient, scale, 32, pow10(32) as i128);",
         "        let (c, s) = (coefficient, scale);")]),
    ("smallest-stage-missing-code-and-spec", [
        (DECIMAL, "        let (coefficient, scale) = strip_zeros(coefficient, scale, 1, POWERS_OF_TEN[1]);\n", ""),
        (PROOFS, "        strip_zeros(c, s, 1, pow10(1) as i128)", "        (c, s)")]),
    ("zero-keeps-its-scale-code-and-spec", [
        (DECIMAL, "            return NormalizedPair {\n                coefficient: 0,\n                scale: 0,\n            };",
         "            return NormalizedPair {\n                coefficient: 0,\n                scale: self.scale,\n            };"),
        (PROOFS, "    if coefficient == 0 {\n        (0, 0)\n    } else {\n        let (c, s) = strip_zeros(coefficient, scale, 32, pow10(32) as i128);",
         "    if coefficient == 0 {\n        (0, scale)\n    } else {\n        let (c, s) = strip_zeros(coefficient, scale, 32, pow10(32) as i128);")]),
    ("normalized-returns-the-retained-scale", [
        (DECIMAL, "        (pair.coefficient, pair.scale)", "        (pair.coefficient, self.scale)")]),
    ("membership-skips-normalization-code-and-spec", [
        (DECIMAL, "        self.admits_normalized(value.normalized_pair())",
         "        self.admits_normalized(NormalizedPair {\n            coefficient: value.coefficient,\n            scale: value.scale,\n        })"),
        (PROOFS, "    admits_normalized(bounds, normalized(retained.0, retained.1))", "    admits_normalized(bounds, retained)")]),
    ("membership-refuses-the-largest-scale-code-and-spec", [
        (DECIMAL, "        if lifted_scale > self.max_scale {", "        if lifted_scale >= self.max_scale {"),
        (PROOFS, "    if lifted_scale > max_scale {\n        false\n    } else if coefficient == 0 {",
         "    if lifted_scale >= max_scale {\n        false\n    } else if coefficient == 0 {")]),
    ("membership-drops-the-upper-bound-code-and-spec", [
        (DECIMAL, "            Some(lifted) => self.lower <= lifted && lifted <= self.upper,",
         "            Some(lifted) => self.lower <= lifted,"),
        (PROOFS, "            i128::MIN <= lifted <= i128::MAX && lower <= lifted <= upper",
         "            i128::MIN <= lifted <= i128::MAX && lower <= lifted")]),
    ("membership-admits-an-overflowing-lift-code-and-spec", [
        (DECIMAL, "            None => false,\n        }\n    }\n}", "            None => true,\n        }\n    }\n}"),
        (PROOFS, "            i128::MIN <= lifted <= i128::MAX && lower <= lifted <= upper",
         "            !(i128::MIN <= lifted <= i128::MAX) || lower <= lifted <= upper")]),
    ("membership-admits-every-zero-code-and-spec", [
        (DECIMAL, "            return self.lower <= 0 && 0 <= self.upper;", "            return true;"),
        (PROOFS, "        lower <= 0 && 0 <= upper\n    } else {\n        let shift = lifted_scale - scale;",
         "        true\n    } else {\n        let shift = lifted_scale - scale;")]),
    ("constructor-admits-every-zero", [
        (DECIMAL, "        if ty.admits(&drawn) {", "        if ty.admits(&drawn) || coefficient == 0 {")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
