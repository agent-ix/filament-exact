#!/usr/bin/env python3
"""Mutants of the u8 tier of the native fraction reduction (FR-408 Behavior 17b,
AC-12 / TC-956).

Each mutant is an exact-text edit of `src/native_fraction.rs` built in a
scratch copy with its own target directory. A mutant is killed when the host
sweep `the_u8_tier_reduces_every_pair_as_euclid_does_across_its_threshold`
fails on it; with `--kani <harness>` the named Kani harness must instead report
`VERIFICATION:- FAILED`.

    python3 scripts/ir258_u8_tier_mutants.py                 # host sweep
    flock /tmp/cargo-slot.lock systemd-run --user --scope -p MemoryMax=24G \\
      -p MemorySwapMax=0 python3 scripts/ir258_u8_tier_mutants.py \\
      --kani native_fraction_gcd8_matches_euclid_over_u8

`--kani-rows [mutant-id ...]` runs the arithmetic mutants (all, or the ids named)
against the Kani CG-domain rows.
Set KANI_RUNNER to a command prefix (the flock, systemd-run and `timeout 1800`
of the lines above) to wrap each Kani run on its own.
"""
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FILE = "src/native_fraction.rs"
RATIONAL = "src/native_rational.rs"
# Mutants inside the gcd8 body, which the Kani gcd8 harness sees; the threshold
# mutant lives in `reduce` and is killed by the host sweep only.
KANI_TIER = {"steps-4", "steps-6", "shift-dropped", "even-parts-kept", "difference-not-odd"}
# Mutants of the arithmetic, killed by the Kani CG-domain rows:
# (harness, mutant id, file, anchor, old, new)
ROWS = [
    ("native_rational_add_cg_domain_matches_reference", "add-forms-minus", RATIONAL,
     "fn form", "_ => first.checked_add(second),", "_ => first.checked_sub(second),"),
    ("native_rational_subtract_cg_domain_matches_reference", "subtract-swapped", RATIONAL,
     "fn form", "Operation::Subtract => first.checked_sub(second),",
     "Operation::Subtract => second.checked_sub(first),"),
    ("native_rational_subtract_cg_domain_from_i64_parts_matches_reference", "subtract-swapped", RATIONAL,
     "fn form", "Operation::Subtract => first.checked_sub(second),",
     "Operation::Subtract => second.checked_sub(first),"),
    ("native_rational_add_is_commutative_over_cg_domain", "add-forms-minus", RATIONAL,
     "fn form", "_ => first.checked_add(second),", "_ => first.checked_sub(second),"),
    ("native_rational_add_is_commutative_from_i64_parts_over_cg_domain", "add-forms-minus", RATIONAL,
     "fn form", "_ => first.checked_add(second),", "_ => first.checked_sub(second),"),
    ("native_rational_multiply_cg_domain_matches_reference", "multiply-forms-sum", RATIONAL,
     "fn form", "Operation::Multiply => a.checked_mul(c)", "Operation::Multiply => a.checked_add(c)"),
    ("native_rational_add_cg_domain_matches_reference", "result-not-reduced", RATIONAL,
     "fn arithmetic(", "reduce(unreduced.0, unreduced.1)", "(unreduced.0, unreduced.1)"),
    ("native_rational_equal_cg_domain_matches_reference", "equality-numerators-only", RATIONAL,
     "pub fn native_rational_equal(", ".map(|()| left == right)",
     ".map(|()| left.numerator == right.numerator)"),
    ("native_rational_equal_all_parts_is_structural", "equality-numerators-only", RATIONAL,
     "pub fn native_rational_equal(", ".map(|()| left == right)",
     ".map(|()| left.numerator == right.numerator)"),
    ("native_rational_order_cg_domain_matches_reference", "order-compares-a-p", RATIONAL,
     "pub fn native_rational_order(",
     "left\n        .numerator\n        .checked_mul(right.denominator)",
     "left\n        .numerator\n        .checked_mul(left.denominator)"),
]
TEST = "the_u8_tier_reduces_every_pair_as_euclid_does_across_its_threshold"

# (id, anchor the edit follows, old text, new text)
MUTANTS = [
    # The worst pairs need 7 steps before the divisor slot holds the gcd, so 16
    # is a margin and 6 is a mutant that some input distinguishes.
    ("steps-4", "fn steps8_16", "steps8_4(steps8_4(steps8_4(steps8_4(state))))",
     "steps8_4(state)"),
    ("steps-6", "fn steps8_16", "steps8_4(steps8_4(steps8_4(steps8_4(state))))",
     "step8(step8(steps8_4(state)))"),
    ("threshold-u16-narrowed", "pub(crate) fn reduce(",
     "match (u8::try_from(n), u8::try_from(d)) {",
     "match (\n        u16::try_from(n).map(|value| value as u8),\n        u16::try_from(d).map(|value| value as u8),\n    ) {"),
    ("shift-dropped", "pub(crate) fn gcd8", "divisor << shift", "divisor"),
    ("even-parts-kept", "pub(crate) fn gcd8",
     "steps8_16((\n        left >> left.trailing_zeros(),\n        right >> right.trailing_zeros(),\n    ))",
     "steps8_16((left, right))"),
    ("difference-not-odd", "fn step8",
     "(small, difference >> difference.trailing_zeros())", "(small, difference)"),
]


def mutate(path, anchor, old, new):
    text = open(path).read()
    at = text.index(anchor)
    head, tail = text[:at], text[at:]
    assert old in tail, f"{anchor}: edit text not found"
    open(path, "w").write(head + tail.replace(old, new, 1))


def run(mutant, kani_harness, file=FILE, qualified="native_rational::kani_proofs"):
    mid, anchor, old, new = mutant
    work = tempfile.mkdtemp(prefix=f"ir258-tier-{mid}-")
    try:
        for entry in ["Cargo.toml", "Cargo.lock", "src", "tests", "clippy.toml", "rustfmt.toml"]:
            src = os.path.join(ROOT, entry)
            (shutil.copytree if os.path.isdir(src) else shutil.copy)(src, os.path.join(work, entry))
        mutate(os.path.join(work, file), anchor, old, new)
        if kani_harness:
            cmd = [*shlex.split(os.environ.get("KANI_RUNNER", "")), "cargo", "kani", "--harness", f"{qualified}::{kani_harness}",
                   "--exact", "--jobs", "1", "--solver", "cadical"]
        else:
            cmd = ["cargo", "test", "--locked", "--test", "ir258_native_rational", TEST]
        proc = subprocess.run(cmd, cwd=work, capture_output=True, text=True,
                              env={**os.environ, "CARGO_TARGET_DIR": os.path.join(work, "target")})
        out = proc.stdout + proc.stderr
        if kani_harness:
            verdict = re.search(r"VERIFICATION:- (\w+)", out)
            return verdict.group(1) if verdict else "NO-VERDICT", "FAILED"
        return ("KILLED" if "test result: FAILED" in out else
                "BUILD-ERROR" if proc.returncode and "test result" not in out else "SURVIVED"), "KILLED"
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    args = sys.argv[1:]
    bad = 0
    if args[:1] == ["--kani-rows"]:
        wanted = set(args[1:])
        for harness, mid, file, anchor, old, new in ROWS:
            if wanted and mid not in wanted:
                continue
            got, want = run((mid, anchor, old, new), harness, file)
            print(f"{harness:70} {mid:22} {got}", flush=True)
            bad += got != want
        sys.exit(1 if bad else 0)
    harness = args[1] if args[:1] == ["--kani"] else None
    for mutant in MUTANTS:
        if harness and mutant[0] not in KANI_TIER:
            continue
        got, want = run(mutant, harness)
        print(f"{mutant[0]:26} {got}", flush=True)
        bad += got != want
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
