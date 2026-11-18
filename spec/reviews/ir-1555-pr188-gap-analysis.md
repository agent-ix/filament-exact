---
id: SR-17940486709564847048057
title: "IR-1555 PR #188 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #188 at 4e54d122; FR-403 Behavior 41, Behavior 42, FR-403-AC-15, FR-403-AC-16, FR-403-AC-17; tests/verus_gate.rs, scripts/ir1555_verus_mutants.py, src/native_decimal/operations.rs, src/native_decimal/operations/verus_proofs.rs, src/native_decimal/operations_tests.rs, tests/ir1508_native_decimal_ops.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-403
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-951
    type: reviews
---

## Summary

Ticket: IR-1555. Gap analysis, planless (Plan completion: not assessed), from `quoin matrix` (quoin 0.29.1) at head.

- FR-403-AC-15 is `tagged-by-ignored-test` through `verus_proofs_verify` (tests/verus_gate.rs:26). FR-403-AC-16 is `tagged-by-ignored-test` through `verus_mutants_are_killed` (tests/verus_gate.rs:45). FR-403-AC-17 is Inspection (`method-without-symbol`), which is correct for an inspection AC. No AC declared Test is untagged, so the #175 pattern does not recur. The old AC-15 (PLANNED, Inspection) is replaced. No stale "not executable" or "waits on a Verus track" text remains in FR-403 or TC-951; the #181 pattern does not recur.
- Every one of the 27 functions Behavior 41 lists has at least one mutant in `scripts/ir1555_verus_mutants.py`, checked one by one against the TC-951 table. The #181 pattern of a covered function without a mutant does not recur. The quality of one mutant's kill is code-review FND-001.
- The charge functions that are not covered (`power_of_ten_bits`, `sbits`, `sdigits`, `sizes`, `expansion`, `expanded`) carry only overflow-freedom postconditions. Behavior 41 excludes them by name. Their values are pinned by the host tests of AC-6, AC-7, AC-9 and AC-14; five host mutants of the rewritten `expanded`/`expansion` and `reduce_quotient` fail those tests.
- No code without an owning requirement: the new code is the ghost proof of Behavior 41 and the two disclosed closure rewrites.

## Verdict

Clean. Each new Test AC is bound to an executable gate test, the gate fails on a broken proof and on a surviving mutant, and the full set of covered functions is mutated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-403-AC-15 | examined | The Verus proofs of Behavior 41 verify with no error (`make verus`) on the production functions Behavior 41 lists. |
| FR-403-AC-16 | examined | Every mutant of `scripts/ir1555_verus_mutants.py` makes a proof of Behavior 41 fail with a failed proof obligation ... A run that only exhausts the solver's resource limit is no verdict and does not count. |
| FR-403-AC-17 | examined | No Kani proof of the six operations or the ordering is claimed, and the Status states what the Verus proof covers and what it does not (Behavior 42). |
| FR-403 Behavior 41 | context_only | A Verus proof SHALL prove the following on the production functions themselves ... Each proof SHALL fail on a mutant of the function it covers. |
