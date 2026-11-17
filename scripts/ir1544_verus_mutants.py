#!/usr/bin/env python3
"""Mutants of the code the Verus proofs cover (FR-408 Behavior 18b, TC-956).

Each mutant is an exact-text edit applied to a scratch copy of `src/`. A mutant
is killed when `scripts/verus.sh` on that copy reports an error. The unmutated
copy must verify first. A mutant whose `spec` entry is not empty also edits the
specification function of the same name the same way, so that the production
function still equals its specification and only the theorem can fail.

    VERUS=/path/to/verus python3 scripts/ir1544_verus_mutants.py [mutant-id ...]
"""
import os
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FRACTION = "native_fraction.rs"
FRACTION_PROOFS = "native_fraction/verus_proofs.rs"
RATIONAL = "native_rational.rs"
NUMERIC = "numeric.rs"

# (id, [(file, old, new), ...])
MUTANTS = [
    ("from-parts-accepts-zero-over-any", [
        (RATIONAL, "} else if !is_canonical_pair(numerator, denominator) {",
         "} else if numerator != 0 && !is_canonical_pair(numerator, denominator) {")]),
    ("from-parts-zero-denominator-late", [
        (RATIONAL, "        if denominator == 0 {\n            Err(NativeRationalRefusal::ZeroDenominator)",
         "        if denominator == 0 && numerator == 0 {\n            Err(NativeRationalRefusal::ZeroDenominator)")]),
    ("step-keeps-even-difference", [
        (FRACTION, "        (small, difference >> difference.trailing_zeros())\n    }\n}\n\n#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::steps_2",
         "        (small, difference)\n    }\n}\n\n#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures r == verus_proofs::steps_2")]),
    ("coprime-both-even-test-wrong", [
        (FRACTION, "    if (left | right) & 1 == 0 {", "    if (left & right) & 1 == 0 {")]),
    ("ladder-64-steps-code-and-spec", [
        (FRACTION, "fn steps_128(state: (u64, u64)) -> (u64, u64) {\n    steps_64(steps_64(state))",
         "fn steps_128(state: (u64, u64)) -> (u64, u64) {\n    steps_64(state)"),
        (FRACTION_PROOFS, "pub open spec fn steps_128(s: (u64, u64)) -> (u64, u64) {\n    steps_64(steps_64(s))",
         "pub open spec fn steps_128(s: (u64, u64)) -> (u64, u64) {\n    steps_64(s)")]),
    ("gcd64-power-of-two-not-restored", [
        (FRACTION, "    divisor << shift\n}\n\n/// Reduce", "    divisor\n}\n\n/// Reduce")]),
    ("gcd-step-odd-case-not-halved-code-and-spec", [
        (FRACTION, "            ((u - v) >> 1, v, g)", "            (u - v, v, g)"),
        (FRACTION_PROOFS, "            (((u - v) as u128) >> 1, v, g)", "            ((u - v) as u128, v, g)")]),
    ("reduce-wide-tier-keeps-denominator-code-and-spec", [
        (FRACTION, "                (n / gcd, d / gcd)", "                (n / gcd, d)"),
        (FRACTION_PROOFS, "        (n / gcd, d / gcd)", "        (n / gcd, d)")]),
    ("reduce-sign-dropped", [
        (FRACTION, "    let numerator = if numerator < 0 {\n        signed.wrapping_neg()\n    } else {\n        signed\n    };\n    (numerator, d as i128)",
         "    let numerator = signed;\n    (numerator, d as i128)")]),
    ("add-forms-minus", [
        (RATIONAL, "                _ => first.checked_add(second),", "                _ => first.checked_sub(second),")]),
    ("capacity-order-swapped", [
        (RATIONAL,
         "            let first = a.checked_mul(q).ok_or(Refuse(Capacity::LeftProduct))?;\n            let second = c.checked_mul(p).ok_or(Refuse(Capacity::RightProduct))?;",
         "            let second = c.checked_mul(p).ok_or(Refuse(Capacity::RightProduct))?;\n            let first = a.checked_mul(q).ok_or(Refuse(Capacity::LeftProduct))?;")]),
    ("result-not-reduced", [
        (RATIONAL, "                let (numerator, denominator) = reduce(unreduced.0, unreduced.1);",
         "                let (numerator, denominator) = unreduced;")]),
    ("normalize-charged-on-numerator-only", [
        (RATIONAL, "        bits(unreduced.0).max(bits(unreduced.1)),", "        bits(unreduced.0),")]),
    ("arithmetic-charge-point-wrong", [
        (RATIONAL, "        ChargePoint::RationalArithmeticArithmetic,", "        ChargePoint::RationalArithmeticNormalize,")]),
    ("order-compares-a-p", [
        (RATIONAL,
         "        .checked_mul(right.denominator)\n        .ok_or(NativeRationalRefusal::Capacity(Capacity::LeftProduct))?;",
         "        .checked_mul(left.denominator)\n        .ok_or(NativeRationalRefusal::Capacity(Capacity::LeftProduct))?;")]),
    ("less-or-equal-strict", [
        (NUMERIC, "            Self::LessOrEqual => ordering.is_le(),", "            Self::LessOrEqual => ordering.is_lt(),")]),
    ("bit-length-off-by-one", [
        (RATIONAL, "(128 - value.unsigned_abs().leading_zeros())", "(127 - value.unsigned_abs().leading_zeros())")]),
]


def verus(src):
    run = subprocess.run([os.path.join(ROOT, "scripts", "verus.sh"), src], capture_output=True, text=True)
    return run.returncode == 0, run.stdout + run.stderr


def main():
    wanted = set(sys.argv[1:])
    scratch = tempfile.mkdtemp(prefix="ir1544-verus-mutants-")
    failures = 0
    try:
        base = os.path.join(scratch, "src")
        shutil.copytree(os.path.join(ROOT, "src"), base)
        ok, out = verus(base)
        if not ok:
            print(out)
            print("the unmutated copy does not verify")
            return 1
        print("unmutated copy: verified")
        for ident, edits in MUTANTS:
            if wanted and ident not in wanted:
                continue
            originals = {}
            for file, old, new in edits:
                path = os.path.join(base, file)
                text = open(path).read()
                originals.setdefault(path, text)
                if text.count(old) != 1:
                    print(f"{ident}: anchor matches {text.count(old)} times in {file}")
                    return 1
                open(path, "w").write(text.replace(old, new))
            ok, out = verus(base)
            for path, text in originals.items():
                open(path, "w").write(text)
            if ok:
                failures += 1
                print(f"{ident}: SURVIVED")
            else:
                reason = next((line for line in out.splitlines() if line.startswith("error")), "error")
                print(f"{ident}: killed ({reason[:70]})")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    print("FAILED" if failures else "all mutants killed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
