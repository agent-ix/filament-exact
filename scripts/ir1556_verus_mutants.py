#!/usr/bin/env python3
"""Mutants of `native_div_rem`, which the Verus proof of the division laws
covers (FR-409 Behavior 24, TC-957).

Each mutant is an exact-text edit applied to a scratch copy of `src/` and is
killed when `scripts/verus.sh` on that copy reports an error. The runner is
the one of `scripts/ir1544_verus_mutants.py`.

    VERUS=/path/to/verus python3 scripts/ir1556_verus_mutants.py [mutant-id ...]
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ir1544_verus_mutants import run  # noqa: E402

INTEGER = "native_integer.rs"

# (id, [(file, old, new), ...])
MUTANTS = [
    ("truncating-does-not-return-on-zero-remainder", [
        (INTEGER, "    if profile == DivisionProfile::Truncating || remainder == 0 {",
         "    if profile == DivisionProfile::Truncating {")]),
    ("floor-shifts-on-same-sign", [
        (INTEGER, "    let (floor_q, floor_r) = if (remainder < 0) != (divisor < 0) {",
         "    let (floor_q, floor_r) = if (remainder < 0) == (divisor < 0) {")]),
    ("floor-shifts-on-dividend-sign", [
        (INTEGER, "    let (floor_q, floor_r) = if (remainder < 0) != (divisor < 0) {",
         "    let (floor_q, floor_r) = if (remainder < 0) != (dividend < 0) {")]),
    ("floor-quotient-not-stepped", [
        (INTEGER, "                Some(q) => q.checked_sub(1),", "                Some(q) => Some(q),")]),
    ("floor-quotient-stepped-up", [
        (INTEGER, "                Some(q) => q.checked_sub(1),", "                Some(q) => q.checked_add(1),")]),
    ("floor-remainder-not-shifted", [
        (INTEGER, "            remainder.wrapping_add(divisor),", "            remainder,")]),
    ("floor-returned-for-euclidean", [
        (INTEGER, "    if profile == DivisionProfile::Floor || floor_r >= 0 {",
         "    if profile != DivisionProfile::Truncating {")]),
    ("euclidean-quotient-not-stepped", [
        (INTEGER, "            Some(q) => q.checked_add(1),\n            None => None,\n        },\n        floor_r.wrapping_sub(divisor),",
         "            Some(q) => Some(q),\n            None => None,\n        },\n        floor_r.wrapping_sub(divisor),")]),
    ("euclidean-remainder-shifted-the-wrong-way", [
        (INTEGER, "        floor_r.wrapping_sub(divisor),", "        floor_r.wrapping_add(divisor),")]),
    ("quotient-kept-for-min-over-minus-one", [
        (INTEGER, "    let quotient = dividend.checked_div(divisor);",
         "    let quotient = if dividend == i128::MIN && divisor == -1 {\n        Some(i128::MIN)\n    } else {\n        dividend.checked_div(divisor)\n    };")]),
]

if __name__ == "__main__":
    sys.exit(run(MUTANTS))
