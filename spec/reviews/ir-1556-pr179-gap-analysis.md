---
id: SR-17940486709564847048048
title: "IR-1556 PR #179 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #179 at 56d9d8b0; IR-1556 ACs 1-2; FR-409 Behavior 24, FR-409-AC-11, FR-409-AC-12 and TC-957 steps 10-11 against src/native_integer.rs, src/native_integer/verus_proofs.rs, scripts/ir1556_verus_mutants.py"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-957
    type: reviews
---

## Summary

Ticket: IR-1556. Plan completion: not assessed (planless). I mapped each ticket AC and each new FR-409 criterion to the code and the evidence behind it.

- IR-1556 AC-1 asks for the FR-409 / TC-957 full-width relation proved in Verus, with a failing mutation. It is delivered by `div_rem_post` on `native_div_rem`, for every `i128` dividend and nonzero divisor under all three profiles. The proof covers the remainder window, the identity `a = q b + r` and the case where the quotient is absent (exactly `MIN / -1`, remainder 0). It has 10 mutants, each killed by a postcondition. I re-ran it: 221 verified, 0 errors, 10 of 10 killed.
- IR-1556 AC-2 asks that the three CG table rows record the Verus verdict. It is not delivered. The PR's ticket comment says "The CG table rows are a CG change", but no follow-up ticket is linked and no deferral is recorded (FND-001).
- FR-409-AC-11 and FR-409-AC-12 are backed by TC-957 steps 10 and 11 and the Verus record. `quoin matrix` reports both as `untagged` with no run evidence: they declare method Test, and no test or trace binder names them (FND-002). FR-408-AC-13 to AC-15 from #175 are in the same state.
- Coverage is limited to `native_div_rem`. The member choice, the result interval, the zero-divisor guard and charging stay with the Kani rows and the host differential. FR-409 Behavior 24 and TC-957 say this, so the scope is not overclaimed. The 20 timeout rows stay as measured rows, and no Kani proof is claimed for them.

## Verdict

The proof part of the ticket is delivered as claimed. The CG part is not, and it has no recorded owner, so IR-1556 must not close on this merge without a deferral. The matrix shows the two new ACs as untagged. That follows the precedent of #175.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | IR-1556 AC-2 ("The three CG table rows record the Verus verdict") is not delivered, and no deferral is recorded. The PR comment calls it "a CG change", but no CG ticket is linked from IR-1556, and nothing in FR-409 or TC-957 says where those rows get the verdict. Either file or link a CG ticket and record the deferral on IR-1556, or keep IR-1556 open after the merge. | Linear IR-1556 description AC-2; Linear IR-1556 comment f268b03e |
| FND-002 | low | FR-409-AC-11 and FR-409-AC-12 declare method `Test (TC-957)`, but `quoin matrix` gives both `static_status: untagged` and "no evidence is bound". The Verus run and the mutant script are not test binders, and no trace names the ACs. Either bind them (a trace on the proof or the script, as the repo's traceability model allows) or declare a method that fits a recorded run, consistently with FR-408-AC-13 to AC-15. | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:146-147 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| IR-1556 AC-1 | examined | FR-409 / TC-957 full-width relation proved in Verus with a failing mutation. |
| IR-1556 AC-2 | examined | The three CG table rows record the Verus verdict. |
| FR-409-AC-11 | examined | The Verus proof of Behavior 24 verifies with no error (`make verus`) on the production function `native_div_rem`. The run is recorded in TC-957's evidence record. The timeout rows of Behavior 19 stay recorded as measured and claim no Kani proof. |
| FR-409-AC-12 | examined | Every mutant of `scripts/ir1556_verus_mutants.py` makes the proof of Behavior 24 fail with a failed proof obligation ... A run that only exhausts the solver's resource limit is no verdict and does not count. |
| TC-957 steps 10-11 and Verus record | examined | `221 verified, 0 errors`; 10 of 10 mutants killed by a failed postcondition, none UNRELIABLE |

## Dispositions

Round 1, fix head 2be4ec49 (reviewed head 56d9d8b0); base still origin/main 16eba89a. Reviewer re-runs in a fresh private worktree under flock /tmp/cargo-slot.lock. `cargo test --locked --test verus_gate -- --ignored`: 2 passed (verus_proofs_verify, verus_mutants_are_killed), 98.7 s wall. Without the ignore flag the two gate tests are skipped (0 passed, 2 ignored), so the default test path is unchanged. Non-tautology probes: the Euclidean window changed to `0 < r` made verus_proofs_verify FAIL (postcondition not satisfied, 220 verified, 1 errors); an extra equivalent mutant in scripts/ir1556_verus_mutants.py made verus_mutants_are_killed FAIL (reviewer-equivalent-survivor: SURVIVED). Both edits were reverted, and git status is clean. `quoin matrix`: FR-408-AC-13, FR-408-AC-14, FR-409-AC-11 and FR-409-AC-12 are tagged-by-ignored-test, and FR-408-AC-15 is tagged. Gates at 2be4ec49: clippy --locked --all-targets -D warnings, make lint, build-no-std, docs and deny pass. fmt-check differs only in tests/qsl2171_undefined_no_element.rs and tests/qsl2326_live_occurrences_limit.rs. The cargo test failure set is the one that fails identically on origin/main: 2 lib equality tests, plus 1 more with test-support; composite_declarations::pairs_without_a_key_stay_unkeyed_and_refuse; union::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences. tests/ir258_native_rational.rs passes 16. The diff 56d9d8b0..2be4ec49 adds no pin and no tool version. The committed spec/reviews/ir-1556-pr179-*.md copies are byte-identical (cmp) to the reviewer's round-0 files.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: FR-409 Status: "Recording the Verus verdict against the three native division rows of the CG tractability table (IR-1556's second criterion) is not part of this change: it is a CG change, deferred to IR-1561." IR-1561 comment bdd934d8 adds that scope; IR-1556 comment e2b328db records the deferral. |
| FND-002 | fixed | after: tests/verus_gate.rs `#[trace("TC-957", "FR-409-AC-11")] ... fn verus_proofs_verify()` and `#[trace("TC-957", "FR-409-AC-12")] ... fn verus_mutants_are_killed()` (both ignored, run the real scripts); FR-408-AC-15 traced on the_tier_harnesses_are_verified_with_covers_and_every_kani_mutant_is_recorded_failing, which now requires the native_rational_equal_all_parts_is_structural VERIFIED row. |
