---
id: SR-17940486709564847047997
title: "IR-1541 PR #171 gap-analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-15
scope: "agent-ix/filament-exact PR #171 at 4002545d, git diff origin/main...HEAD; IR-1541; src/native_integer.rs, src/native_integer/division_proofs.rs, tests/ir1541_native_integer_divide.rs, spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md, spec/test-cases/TC-957-native-integer-divide-modulo-plan.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
---

## Summary

Ticket: IR-1541. This is a planless gap analysis; plan completion was not assessed. `quire matrix --scope .` at 4002545d gives these FR-409 rows:

- AC-1 to AC-4: `tagged` (tests/ir1541_native_integer_divide.rs:227, 296, 394 and 436, 547).
- AC-5, AC-6, AC-8 and AC-10: `method-without-symbol` (Inspection criteria).
- AC-7: `untagged`.
- AC-9: `tagged-by-ignored-test` (the `#[ignore]` replay driver at :615).

All new production code is owned by FR-409: the two operation variants, the capacity helpers, `plan_division`, the schedule arm and the `cfg(kani)` harness module. No stub, `todo!` or `unimplemented!` exists outside deliberate `unreachable!` arms for the two new kinds in the four-operation reference and tests. No test asserts a constant against itself. The differential compares against `divide`/`modulo` over `Integer`, the plan-free evaluators and independent literals.

The evidence defects (replay labels, the missing `modulo_const_neg1_w128` mutant run, the probe records) are recorded once, in SR-17940486709564847047998 (FND-001 to FND-003) and SR-17940486709564847047996 (FND-001). They are not repeated here.

## Verdict

One traceability gap: AC-7 has no tagged evidence. Otherwise the code-to-requirement trace is complete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-409-AC-7 (Test, installed Kani) is `untagged` in the matrix. The ladder and runner scripts produce the evidence, but no test or tagged check binds them to AC-7, and AC-9 is reached only through an `#[ignore]` driver. Tag an evidence check for AC-7 (for example, a test that parses TC-957's evidence record and asserts one terminal state per harness, with time and peak for every executed row and the probe rows present), or record in TC-957 why AC-7 stays a manual Kani step. | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:141, scripts/ir1541_ladder.py:1-68 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-409-AC-1..AC-4 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:135-138 | matrix rows `tagged`, bound to tests/ir1541_native_integer_divide.rs |
| FR-409-AC-5, AC-6, AC-8, AC-10 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:139-144 | Inspection criteria, `method-without-symbol` |
| FR-409-AC-7 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:141 | The probe of Behavior 17 ran before the full set, and its two records exist with time and peak. ... |
| FR-409-AC-9 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:143 | Each mutant of Behavior 22 fails the host differential at its distinguishing input ... |

## Dispositions

Disposition pass 1, against fix commit 7eac59d1. `quire matrix --scope .`: FR-409-AC-7 is `tagged` (tests/ir1541_native_integer_divide.rs:652), and FR-409-AC-9 is `tagged` (:615 ignored driver and :652). The new test passes at 7eac59d1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `the_evidence_record_settles_every_harness_with_its_probe_and_mutant` is tagged FR-409-AC-7 and FR-409-AC-9. It parses TC-957 and asserts one record per expected harness id (80), time and peak for VERIFIED and timeout rows, a FAILED assigned mutant without REPLAY-SAME on every VERIFIED row, valid 'covered by' targets, and both probe rows. |
