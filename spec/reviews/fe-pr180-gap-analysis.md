---
id: SR-17940486709564847048048
title: "filament-exact PR #180 gap-analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #180 at d436f9c0, git diff origin/main...d436f9c0; QSL-2173; src/equality.rs, tests/composite_declarations.rs, tests/union.rs against FR-375-AC-9/10, FR-376-AC-4, FR-377-AC-6, FR-379-AC-7, FR-410-AC-1"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-375
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-376
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-377
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-379
    type: reviews
---

## Summary

Ticket: QSL-2173. A planless gap analysis of the PR's production change (the slot-arm pair counting in `plan_pairs_with_relation`) and its two test edits, checking which criteria each test asserts. Plan completion: not assessed. The criteria-to-tests check was done by hand: I read the tagged tests and ran mutants, and did not use the `quoin matrix` output.

The mutants were applied one at a time to `src/equality.rs` at d436f9c0 and run with `cargo test --features test-support`:

- Drop the pair added in the `Absent/Absent | Null/Null` arm: KILLED by the three E21 qualification tests (FR-375-AC-9/10, TC-932).
- Add 1 to the `MapKeys` arm, which brings back #173's double count: KILLED by `map_equality_matches_entries_by_key_and_checks_map_type_admission` (FR-410-AC-1, 5 pairs).
- Drop the pair added in the mismatched-slot arm (`Present/_`, `Absent/_`, `Null/_` against a different kind): SURVIVED. Every lib and integration test passes (FND-001).

Bisects: `d99c707^` passes and `d99c707` fails the three E21 tests (16 against 17 pairs). `85034c5` passes and `0fe3d91` fails both float integration tests. Both root causes are as the author states.

## Verdict

PASS with one medium gap. The restored count is spec-backed: FR-376-AC-4 keeps the pre-requirement counts, and FR-375-AC-9/10 and TC-932 say 17. The Absent/Null arm and the MapKeys arm are pinned by tests, but the mismatched-slot arm the PR also changes is not pinned by any test. One low pre-existing note on the FR-375 E21 description.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test pins the pair count of a mismatched slot pair (for example `Present` against `Absent`, or `Absent` against `Null`). Removing the `pairs.add` this PR restores in that arm (src/equality.rs:244-248) leaves the whole suite green. That is how #173 dropped it silently, and the same could happen again. FR-376-AC-4 ('the pair counts are those of the same plan before this requirement') covers it. Add a test asserting `pair_events()` for two one-declaration composites differing only by `Present(x)`/`Absent` and by `Absent`/`Null` in one slot (each 1 + slot count). | src/equality.rs:244-248 |
| FND-002 | low | Pre-existing, outside the diff: FR-375's statement describes the E21 input as 'two separately constructed depth-four `CollectionKind::Sequence` values each have 16 visited occurrences'. The test fixture `e21_list_shape_operands` builds eight-node composite linked lists (head `Integer`, tail slot `Present`/`Absent`). The count of 17 agrees with it, but the shape does not, so a reader cannot derive 17 from the spec text. | spec/functional/FR-375-composable-equality-effect-seam.md:21 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-375-AC-9 | examined | spec/functional/FR-375-composable-equality-effect-seam.md:32 | On equal and first-head-unequal bounded kernel operands, `planned_equality_with_plan` returns the Boolean relation and 17 planned pair events from the same formed plan |
| FR-375-AC-10 | examined | spec/functional/FR-375-composable-equality-effect-seam.md:33 | When `Meter::charge` refuses the 17th pair, the result-bearing kernel operation returns the typed stop with exactly the prior 16 pair charges admitted |
| TC-932 expected | examined | spec/test-cases/TC-932-kernel-equality-plan-and-admitted-pairs.md:22 | each reports 17 planned pairs from one plan and admits 17 pair charges on success |
| FR-376-AC-4 | examined | spec/functional/FR-376-composite-equality-across-declarations.md:47 | The pair counts are those of the same plan before this requirement. |
| FR-410-AC-1 map pair count | examined | src/equality.rs:500-530 | plan_equality(&first, &reordered).unwrap().pair_events() == 5 |
| FR-377-AC-6 | examined | spec/functional/FR-377-composite-canonical-key-across-declarations.md:51 | the distinct float pair instead forms through `form_grouped` and `form` |
| FR-379-AC-7 | examined | spec/functional/FR-379-carry-a-union-member-and-its-positional-payload-as-a-kernel-value.md:59 | A union with an IEEE payload has no `compare_keys` key, forms through `form_grouped` for a `Set` |

## Dispositions

Round 1, verified against fix commit 13f6a73a.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The new test `absent_null_and_mismatched_slot_pairs_count_one_pair_each` (TC-924, FR-376-AC-4) asserts `pair_events == 3` for Absent/Absent, Null/Null, Present/Absent and Absent/Null tail slots. Re-run mutants: removing the mismatched-slot add now fails this test (it survived in round 0), and removing the Absent/Null add fails it together with the three E21 tests. |
| FND-002 | fixed | FR-375 line 21 now describes eight-node composite lists (head `Integer` slot, tail slot, last tail `Absent`): 16 value pairs plus the Absent/Absent tail pair = 17. |
