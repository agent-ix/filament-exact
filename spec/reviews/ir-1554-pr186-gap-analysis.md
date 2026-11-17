---
id: SR-17940486709564847048054
title: "IR-1554 PR #186 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #186 at c7a4541e; FR-402 Behavior 11, FR-402-AC-13, FR-402-AC-14, FR-402-AC-15; tests/verus_gate.rs, src/native_decimal/tests.rs, scripts/ir1554_verus_mutants.py, src/native_decimal.rs, src/native_decimal/verus_proofs.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-402
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-950
    type: reviews
---

## Summary

Ticket: IR-1554. Gap analysis, planless (Plan completion: not assessed), from `quoin matrix` (quoin 0.29.1) at head.

- FR-402-AC-13 is `tagged-by-ignored-test` through `verus_proofs_verify` (tests/verus_gate.rs:28). FR-402-AC-14 is `tagged-by-ignored-test` through `verus_mutants_are_killed` (tests/verus_gate.rs:46). FR-402-AC-15 is `tagged` through `the_power_table_holds_every_power_of_ten_inside_i128` (src/native_decimal/tests.rs:137). No AC declared Test is untagged; the #175 pattern does not recur.
- The AC-15 test does not re-read the table. It computes `expected` from 1 by `checked_mul(10)`, compares all 39 entries, and checks that `10^39` overflows. With `table[17] += 1` added to the const initializer, the test fails at tests.rs:142 ("10^17").
- No other table carries the proof. `POWERS_OF_TEN` is the only constant the proof reads. src/native_decimal/operations.rs also uses it, but operations.rs is outside the Verus scope and Behavior 11 does not claim it.
- Every function Behavior 11 lists has a mutant: `from_bindings`, `admits`, `admits_normalized`, `normalized`, `normalized_pair` and `strip_zeros`. The #181 pattern of a covered function without a mutant does not recur.
- The code-and-spec mutants edit both the code and its spec copy. The #179 code-only pattern does not recur.

## Verdict

Clean. Each new AC is bound to a test that exercises it. The assumption test is independent of the table and catches a corrupted entry.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-402-AC-13 | examined | The Verus proof of Behavior 11 verifies with no error (`make verus`) on the production functions Behavior 11 lists. |
| FR-402-AC-14 | examined | Every mutant of `scripts/ir1554_verus_mutants.py` makes the proof of Behavior 11 fail with a failed proof obligation ... A run that only exhausts the solver's resource limit is no verdict and does not count. |
| FR-402-AC-15 | examined | The table `POWERS_OF_TEN`, which the proof of Behavior 11 assumes, holds `10^0` through `10^38`: a host test compares every entry with a power computed by repeated checked multiplication, and checks that `10^39` leaves signed 128 bits. |
| FR-402 Behavior 11 | context_only | A Verus proof SHALL prove the following on the production functions themselves ... The proof SHALL fail on a mutant of each function it covers. |
