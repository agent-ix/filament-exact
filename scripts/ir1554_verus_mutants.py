#!/usr/bin/env python3
"""Mutants of the native Decimal constructor and normalized view the Verus
proofs cover (FR-402 Behavior 11, TC-950).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports a failed proof obligation.
A mutant that also edits the specification function of the same name leaves
the production function equal to its specification, so only a theorem can
fail. The runner is the one of `scripts/ir1544_verus_mutants.py`.

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
    ("strip-ignores-the-scale", [
        (DECIMAL, "    if scale >= places && coefficient % power == 0 {", "    if coefficient % power == 0 {")]),
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
    ("zero-keeps-its-scale", [
        (DECIMAL, "            return NormalizedPair {\n                coefficient: 0,\n                scale: 0,\n            };",
         "            return NormalizedPair {\n                coefficient: 0,\n                scale: self.scale,\n            };")]),
    ("membership-refuses-the-largest-scale", [
        (DECIMAL, "        if lifted_scale > self.max_scale {", "        if lifted_scale >= self.max_scale {")]),
    ("membership-drops-the-upper-bound", [
        (DECIMAL, "            Some(lifted) => self.lower <= lifted && lifted <= self.upper,",
         "            Some(lifted) => self.lower <= lifted,")]),
    ("membership-admits-an-overflowing-lift", [
        (DECIMAL, "            None => false,\n        }\n    }\n}", "            None => true,\n        }\n    }\n}")]),
    ("membership-admits-every-zero", [
        (DECIMAL, "            return self.lower <= 0 && 0 <= self.upper;", "            return true;")]),
    ("constructor-admits-every-zero", [
        (DECIMAL, "        if ty.admits(&drawn) {", "        if ty.admits(&drawn) || coefficient == 0 {")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
