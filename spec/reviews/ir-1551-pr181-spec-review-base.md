---
id: SR-17940486709564847048052
title: "IR-1551 PR #181 FR-405 and TC-953 spec review"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #181 at 182ab1ce; spec/functional/FR-405-kani-tractable-native-float-carrier.md (Behavior 22, Behavior 23, AC-15, AC-17, AC-18, Status), spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md (Description, steps 11-12, Verus record, Status), docs/verus-proofs.md; compared with filament-contract-codegen origin/main docs/qualification/ir1530-general-route-scalar-table.json (read-only)"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-953
    type: reviews
---

## Summary

Ticket: IR-1551. Spec review of the FR-405 and TC-953 edits.

- The twelve rows match. The CG table on origin/main has exactly twelve native IEEE `not_tractable` rows, each with "owner IR-1551": Divide Binary32 NearestEven and Exact; SquareRoot Binary32 NearestEven; FusedMultiplyAdd Binary32 NearestEven; Multiply Binary64 NearestEven; Divide Binary64 NearestEven, Exact and directed; SquareRoot Binary64 NearestEven and Exact; FusedMultiplyAdd Binary64 NearestEven and Exact. TC-953's grouping (Divide five, SquareRoot three, Multiply one, FusedMultiplyAdd three) is the same set.
- Both widths and all six spellings are covered for real. The row theorems take `fmt` with `wf(fmt)` (binary32 or binary64) and `rounding` free, so no width or spelling is left out.
- Behavior 23 states its exclusions: classify, NaN propagation, charges, the strict `Exact` refusal and the composition inside `arithmetic`. TC-953 says the same.
- Behavior 22 and AC-15 now both say "No Kani proof", so they agree (#179's Behavior-vs-AC pattern is not repeated).
- AC-17 and AC-18 are atomic, so #175 and #179's compound-AC pattern is not repeated.
- The Verus record gives only a count and wall times: no tool version, no pin, no "pinned release".
- The deferral of the CG half has an owned ticket, IR-1562. Its acceptance criteria require each row to say what is not covered.
- The trusted-base list in Behavior 23 is incomplete again (FND-003). This is the #179 spec FND-002 pattern.

## Verdict

Behavior 23 itself is accurate. Medium finding: the FR-405 Status calls the Verus proof the twelve rows' "deductive verdict" without saying, per row, what those rows check that the proof does not cover. Two low findings: stale "no float Verus ticket" text in TC-953, and an unnamed library trusted base.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-405 Status says of the twelve timeout rows: "Their deductive verdict is the Verus proof of Behavior 23 ... Divide, SquareRoot, Multiply and FusedMultiplyAdd are proved for both widths and every rounding spelling at the level Behavior 23 states." Each row's harness checks more than the proof covers: the NaN result (sign, quietness, payload), the `invalid` flag, that only `Exact` refuses, provenance and the result-unit charge. For the four `Exact` rows (Divide Binary32 and Binary64, SquareRoot Binary64, FusedMultiplyAdd Binary64), the strict refusal is the property under test, and Behavior 23 excludes it. "Deductive verdict" therefore claims more than was proved. IR-1562 will record CG rows from this text. State, per row or per row class, which harness outputs stay open (NaN and invalid, refusal, provenance, charge, and the `arithmetic` composition), and say that the Verus proof does not stand in for a VERIFIED harness. | spec/functional/FR-405-kani-tractable-native-float-carrier.md:114 |
| FND-002 | low | TC-953 still says twice that the timeout rows wait on "a deductive prover (no float Verus ticket exists yet; planner to assign)": in the record's "What changed" paragraph and in the Status paragraph. The PR adds the Verus record for IR-1551 right below, so the text contradicts it. Replace the remark with IR-1551 and the Verus record. | spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:123; spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:211 |
| FND-003 | low | Behavior 23 says the proof "assumes the prover library's specifications of the `core` integer functions used" and names none of them. The proof relies on the library specifications of `u64::leading_zeros` (and the library lemma `axiom_u64_leading_zeros`), `u128::is_multiple_of` (the divide sticky), `u128::from` and the integer shifts. It also relies on `u128::leading_zeros` from src/verus_std_specs.rs. Name them, as FR-409 Behavior 24 does after #179 FND-002. | spec/functional/FR-405-kani-tractable-native-float-carrier.md:156 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-405 Behavior 23 | examined | A Verus proof SHALL prove the following on the production functions themselves, for both widths, all six rounding spellings and every operand, with no bound. (a) `round_at` returns the multiple of its grid ... It does not cover `classify` and NaN propagation, the charge sequence, the strict `Exact` refusal or the composition of these functions inside `arithmetic` ... |
| FR-405 Behavior 22 | examined | The kernel SHALL claim no Kani proof for a combination whose harness has no executed VERIFIED verdict ... A deductive proof of such a combination is Behavior 23 and is recorded separately. |
| FR-405-AC-15 | examined | No Kani proof is claimed for a combination whose harness has no executed VERIFIED verdict, and the Status records, per operation and width, the verdict or the table row. |
| FR-405-AC-17 | examined | The Verus proofs of Behavior 23 verify with no error (`make verus`) on the production functions Behavior 23 lists. |
| FR-405-AC-18 | examined | Every mutant of `scripts/ir1551_verus_mutants.py` makes a proof of Behavior 23 fail with a failed proof obligation ... A run that only exhausts the solver's resource limit is no verdict and does not count. |
| FR-405 Status (IR-1551 paragraph) | examined | Their deductive verdict is the Verus proof of Behavior 23 (IR-1551, AC-17 and AC-18, ...): Divide, SquareRoot, Multiply and FusedMultiplyAdd are proved for both widths and every rounding spelling at the level Behavior 23 states. |
| TC-953 steps 11-12, Verus record, Status | examined | `make verus` (the whole crate) `393 verified, 0 errors` 7.0 s wall; 17 of 17 mutants killed by a failed proof obligation, none UNRELIABLE |
| CG ir1530-general-route-scalar-table.json native IEEE rows | context_only | twelve native rows with verdict not_tractable, provenance "waits on the deductive lane, owner IR-1551" |
| IR-1562 | context_only | CG: record the Verus verdict for the twelve float not_tractable rows in the tractability table |
