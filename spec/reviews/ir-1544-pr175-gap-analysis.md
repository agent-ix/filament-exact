---
id: SR-17940486709564847048045
title: "IR-1544 PR #175 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #175 at e6624e2a; IR-1544 ACs 1-3 and the IR-1544 scope note; FR-408 Behavior 18a, FR-408-AC-11, FR-408-AC-13 and TC-956 step 12 against src/native_fraction/verus_proofs.rs, src/native_rational/verus_proofs.rs, src/native_rational/kani_proofs.rs, scripts/ir1544_verus_mutants.py"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-956
    type: reviews
---

## Summary

Ticket: IR-1544. Plan completion: not assessed (planless). Each ticket AC and each FR-408 criterion the PR touches is mapped to the code and evidence that back it.

- IR-1544 AC-1 (Verus proof of `from_i64_parts` gcd-equivalence over full i64, checked in CI or a recorded run). Delivered by `row_from_i64_parts` with `lemma_is_canonical_pair` and `lemma_coprime`. The production spec is attached to `NativeRational::from_i64_parts`. The run is recorded in TC-956 and was re-run by the reviewer: 208 verified, 0 errors.
- IR-1544 AC-2 (the 6 full-i64 rows reference the Verus proof). TC-956 has a row-to-theorem table, and the FR-408 Status names it. Equality is split into a Verus lemma (`row_equal_structural_is_cross_product`) and a Kani harness on the production function (`native_rational_equal_all_parts_is_structural`). The split is stated honestly. The arithmetic and ordering rows do not prove that a sufficient Meter completes, and TC-956 says so.
- IR-1544 AC-3 (IR-258 AC-11 closes against it). FR-408-AC-11 was rewritten from PLANNED to executed with (a) and the full-i64 rows routed to AC-13. See the spec-review SR for the Behavior 18 consistency issue.
- The IR-1544 scope note, added on the planner's ruling, says the native Divide/Modulo relation from IR-1541 "belongs to this Verus lane" and that the `apply_law` split "lands with this ticket's work". The PR does not touch it (FND-001).

## Verdict

The six rows are delivered as claimed, and the trusted base is listed. One gap: the scope the planner added to IR-1544 is not in this PR, so the ticket cannot close on this merge without an explicit deferral.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | IR-1544's scope note (planner ruling, "no new ticket") adds the native Divide/Modulo relation from IR-1541 to this Verus lane: the `apply_law` split, five Kani harness rows, and the two Verus steps `a = q0*b + r0` and `a - (q0+k)*b = r0 - k*b`. It says this "lands with this ticket's work". PR #175 contains none of it, and neither the PR nor TC-956 states a deferral. Either deliver it in this PR or record on IR-1544 that it lands in a second PR under the same ticket, and keep IR-1544 open after this merge. | Linear IR-1544 comment 1c2853da |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| IR-1544 AC-1 | examined | Verus proof of from_i64_parts gcd-equivalence over the full i64 domain, checked in CI or a recorded run. |
| IR-1544 AC-2 | examined | The 6 full-i64 rows of IR-258 reference the Verus proof as their executed verdict. |
| IR-1544 AC-3 | examined | IR-258 AC-11 closes against it. |
| IR-1544 scope note (Divide/Modulo) | examined | the native Divide/Modulo relation from IR-1541 belongs to this Verus lane ... it lands with this ticket's work, not before. |
| FR-408-AC-13 | examined | The Verus proofs of Behavior 18a verify with no error under the pinned Verus release (`make verus`) ... every mutant of `scripts/ir1544_verus_mutants.py` fails them; and the Kani harness `native_rational_equal_all_parts_is_structural` is VERIFIED with its covers reached and fails its mutant. |
| FR-408-AC-11 | examined | The Kani harnesses of Behavior 18 (b over the CG domain, c) return an executed terminal VERIFIED verdict ... Behavior 18 (a) and the full-`i64` rows of Behavior 18 (b) are measured not-tractable rows under Kani and claim no Kani proof; their executed verdict is the Verus proof of Behavior 18a (FR-408-AC-13). |
| TC-956 step 12 | examined | For AC-13, run `make verus` ... run `make verus-mutants` ... run the Kani harness `native_rational_equal_all_parts_is_structural` ... and its mutant (`equality-numerators-only`), and record both. |
