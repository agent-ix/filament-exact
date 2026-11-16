---
id: SR-17940486709564847048018
title: "IR-1446 / IR-258 PR #172 gap-analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-15
scope: "agent-ix/filament-exact PR #172 at 5098e906, git diff origin/main...HEAD; IR-1446, IR-258; spec/functional/FR-405-kani-tractable-native-float-carrier.md, spec/functional/FR-408-kani-tractable-native-rational-carrier.md, tests/ir1446_native_float.rs, tests/ir258_native_rational.rs, src/native_fraction.rs, src/native_float/kani_proofs.rs, src/native_rational/kani_proofs.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
---

## Summary

Tickets: IR-1446, IR-258. This is a planless gap analysis; plan completion was not assessed. `quire matrix --scope .` at 5098e906 gives these rows:

- FR-405: AC-1 to AC-6, AC-8 and AC-10 to AC-13 are `tagged`. AC-7, AC-9, AC-14 and AC-15 are `method-without-symbol` (Inspection).
- FR-408: AC-1 to AC-9 and AC-12 are `tagged`, and AC-10 is `method-without-symbol` (Inspection). AC-11 is `untagged`; it is PLANNED and claims no credit.

The new production code is `gcd8`/`step8` and the tier branch in `reduce`, owned by FR-408 Behavior 17b. It is exercised by the tagged AC-12 host sweep and by the AC-9 differential. The float side adds no production code; the harness changes are owned by FR-405 Behavior 19. No stub, `todo!` or `unimplemented!` was found. The new tests compare against Euclid or the TC tables, never against the body under test.

## Verdict

No AC claims credit beyond its executed evidence at the matrix level. Two low traceability gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-405 Behavior 19 gained a new SHALL: hold the outcome in `ManuallyDrop`, decide by IEEE rule any region where the oracle's model is wrong, and record the fault in TC-953. No acceptance criterion carries it. The new test `the_tool_fault_rows_name_harnesses_that_exist_and_are_not_kernel_failures` asserts that sentence's TC-953 'Tool faults' table but is tagged `FR-405-AC-11`, which is about a per-combination verdict or measured row. Either extend AC-12 (or add an AC) to cover the rule-decided regions and the recorded tool faults and retag the test, or drop the tag. | tests/ir1446_native_float.rs:1069-1082, spec/functional/FR-405-kani-tractable-native-float-carrier.md:68 |
| FND-002 | low | FR-408-AC-12 has a Kani half: 'a Kani harness proves the tier body equal to `u8` Euclid ... with its cover reached', and 'each mutant ... fails the host sweep or the Kani harness'. Only the host sweep is tagged. Nothing automated checks TC-956's tier rows or the Kani mutant record. FR-405 has evidence-parsing tests for TC-953 that do this; FR-408 has none. Add a small TC-956 evidence test (tier rows present, VERIFIED, covers N of N), or say in AC-12 that the Kani half is evidenced by the record. | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:114, tests/ir258_native_rational.rs:1027 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-405-AC-11 | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:87 | For each combination of operation and width, either a Kani harness of Behavior 19 returns an executed terminal VERIFIED verdict, or a measured row exists ... |
| FR-405-AC-12 | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:88 | Each VERIFIED harness has a vacuity cover that is reached and a harness mutant that it fails ... |
| FR-405-AC-13 | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:89 | The peak memory of every executed harness ... is at most the 16 GiB ceiling |
| FR-408-AC-10 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:112 | the reduction is the width-tiered reduction of Behaviors 17a and 17b ... |
| FR-408-AC-11 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:113 | PLANNED: the Kani harnesses of Behavior 18 (a, b over the CG domain, c) return an executed terminal VERIFIED verdict ... |
| FR-408-AC-12 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:114 | The `u8` tier of the reduction (Behavior 17b) agrees with Euclid's algorithm on every pair ... |
| FR-405-AC-1..AC-10, AC-14, AC-15; FR-408-AC-1..AC-9 | context_only | spec/functional/ | matrix rows unchanged by this PR |
