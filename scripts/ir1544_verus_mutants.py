#!/usr/bin/env python3
"""Mutants of the code the Verus proofs cover (FR-408 Behavior 18b, TC-956).

Each mutant is an exact-text edit applied to a scratch copy of `src/`. A mutant
is killed when `scripts/verus.sh` on that copy reports a failed proof
obligation (a postcondition, precondition or assertion that does not hold, or
a possible overflow). A run that only exhausts the solver's resource limit is
no verdict and is reported as UNRELIABLE, which fails the script. The
unmutated copy must verify first. A mutant whose `spec` entry is not empty also edits the
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

# The errors that are a verdict: a proof obligation Verus checked and found
# not to hold. A resource-limit error is not one.
SEMANTIC = (
    "error: postcondition not satisfied",
    "error: precondition not satisfied",
    "error: requires not satisfied",
    "error: assertion failed",
    "error: possible arithmetic underflow/overflow",
    "error: possible division by zero",
)

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
        (FRACTION, "    proof! { reveal(verus_proofs::steps_128); }\n    steps_64(steps_64(state))",
         "    proof! { reveal(verus_proofs::steps_128); }\n    steps_64(state)"),
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
    ("gcd256-or-instead-of-sum", [
        (FRACTION, "    g * (u + v)\n}", "    g * (u | v)\n}")]),
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


def run(mutants):
    """Run `mutants` (all, or the ids named on the command line); 0 if every one is killed."""
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
        for ident, edits in mutants:
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
            verdict = [line for line in out.splitlines() if line.startswith(SEMANTIC)]
            if ok:
                failures += 1
                print(f"{ident}: SURVIVED")
            elif "is not supported" in out or "error[E" in out:
                failures += 1
                print(f"{ident}: INVALID (the mutant does not build under Verus)")
            elif not verdict:
                # A solver that ran out of resources gave no verdict; like a
                # Kani timeout, that claims nothing.
                failures += 1
                print(f"{ident}: UNRELIABLE (no failed proof obligation, only a resource limit)")
            else:
                print(f"{ident}: killed ({verdict[0]})")
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    print("FAILED" if failures else "all mutants killed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(run(MUTANTS))
