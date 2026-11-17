---
id: SR-17940486709564847048049
title: "filament-exact PR #180 spec-review (base)"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #180 at d436f9c0, git diff origin/main...d436f9c0; QSL-2173; spec/functional/FR-377-composite-canonical-key-across-declarations.md, spec/functional/FR-379-carry-a-union-member-and-its-positional-payload-as-a-kernel-value.md, spec/test-cases/TC-925-composite-canonical-key-across-declarations.md; float-key sweep of spec/"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-377
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-379
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-925
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-393
    type: reviews
---

## Summary

Ticket: QSL-2173. Base spec review of the four spec edits in PR #180 (FR-377 Behavior 6 and AC-6, TC-925 expected result 6, FR-379-AC-7), plus a sweep of `spec/` for float-key text that #174 (0fe3d91, which changed no spec) left stale. The sub-analyses (EARS, integrity and the rest) were not run separately: the change only rewords existing criteria and adds no requirement, object or relationship.

The cited criterion is real. QSpec FR-149-AC-26 (filament-specification `spec/functional/type-model/FR-149-apply-complete-equality-matrix.md:446`) gives float-set equality by bit identity (`set[+0, -0] = set[-0, +0]` is true), so a two-member float set must form. Two other criteria back this more directly: FR-144-AC-6 (a set element type holding a float is admitted and keyed by the FR-148 `totalOrder` key) and FR-144-AC-25 (the visiting order of a `Set<Float64>` is the `totalOrder` key order). The edits contain no pins or tool versions.

The edits are accurate as far as they go. FR-377 Behavior 6 correctly says "ordered by its total-order membership key". AC-6 and TC-925 step 6 keep the composite/integer and mismatched-slot refusals, and `compare_keys` stays `None` for floats (Behavior 4).

## Verdict

FAIL on consistency. The PR fixes FR-377, TC-925 and FR-379-AC-7, but FR-379's own Behavior 7 and TC-927's expected result 7 still say the IEEE-union `Set` refuses `CanonicalKeyUnavailable`, which now contradicts both the edited AC and the tests (FND-001, FND-002). FR-393 still says ordinary equality refuses floats, and the test tagged FR-393-AC-4 asserts the opposite (FND-003; #174 left this stale, but it has the same root cause). One low wording/citation finding (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-379 Behavior 7 still says "A union that holds an IEEE value has no key: a `Set` formed through `form_grouped` refuses `CheckedInvariant { cause: CanonicalKeyUnavailable }`". This contradicts the edited FR-379-AC-7 (the set forms, ordered by the membership key) in the same file. | spec/functional/FR-379-carry-a-union-member-and-its-positional-payload-as-a-kernel-value.md:45 |
| FND-002 | medium | TC-927 expected result 7 still reads "`CanonicalKeyUnavailable` for the `Set`" for the `form_grouped` IEEE-union set of step 7. This contradicts FR-379-AC-7 as edited and the updated `ieee_union_payload_has_no_key_and_sequence_keeps_occurrences`. | spec/test-cases/TC-927-union-carrier.md:36 |
| FND-003 | medium | FR-393 still says membership "SHALL NOT expand ordinary equality eligibility" (line 15), "Ordinary equality SHALL retain its existing planner and IEEE eligibility" (line 30) and, in FR-393-AC-4, "ordinary equality still refuses direct and nested float operands". Since #174, `plan_equality` admits direct and nested floats (QSpec FR-149-AC-25), and `membership_keeps_typed_malformed_refusals_and_float_equality_is_available` (src/collection.rs:1259), tagged FR-393-AC-4, asserts `plan_equality(..).is_ok()`: the test contradicts the AC it traces to. Outside this diff, but it is the same stale-float-spec root cause this PR exists to clear. | spec/functional/FR-393-collection-float-membership.md:15,30,39 |
| FND-004 | low | FR-377-AC-6, TC-925 expected result 6 and FR-379-AC-7 say the members are kept "in bit-identity membership order". Bit identity is an equivalence, not an order. The order is the FR-148 `totalOrder` key (QSpec FR-144 key table, FR-144-AC-6/AC-25), as FR-377 Behavior 6 already says. Reword to "total-order membership key order" and cite FR-144-AC-6/AC-25 next to FR-149-AC-26. | spec/functional/FR-377-composite-canonical-key-across-declarations.md:51 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-377 Behavior 6 | examined | spec/functional/FR-377-composite-canonical-key-across-declarations.md:37 | Float membership instead uses bit identity (FR-393) and a float member is ordered by its total-order membership key (QSpec FR-149-AC-26), so a distinct float pair forms through `form` and `form_grouped` with both members retained in that order. |
| FR-377-AC-6 | examined | spec/functional/FR-377-composite-canonical-key-across-declarations.md:51 | the distinct float pair instead forms through `form_grouped` and `form`, both members retained in bit-identity membership order. |
| FR-377 Behavior 4 | context_only | spec/functional/FR-377-composite-canonical-key-across-declarations.md:35 | `compare_keys` still returns `None` ... and for a `Float`. |
| TC-925 expected 6 | examined | spec/test-cases/TC-925-composite-canonical-key-across-declarations.md:39 | The `[f1, f2]` formation completes through both, retaining two members in bit-identity membership order. |
| FR-379-AC-7 | examined | spec/functional/FR-379-carry-a-union-member-and-its-positional-payload-as-a-kernel-value.md:59 | A union with an IEEE payload has no `compare_keys` key, forms through `form_grouped` for a `Set` ordered by its bit-identity membership key (QSpec FR-149-AC-26), and is kept by a `Sequence`. |
| FR-379 Behavior 7 | examined | spec/functional/FR-379-carry-a-union-member-and-its-positional-payload-as-a-kernel-value.md:45 | a `Set` formed through `form_grouped` refuses `CheckedInvariant { cause: CanonicalKeyUnavailable }` |
| TC-927 expected 7 | examined | spec/test-cases/TC-927-union-carrier.md:36 | `CanonicalKeyUnavailable` for the `Set` and a kept pair for the `Sequence`. |
| FR-393 Behavior, AC-4 | examined | spec/functional/FR-393-collection-float-membership.md:15-39 | ordinary equality still refuses direct and nested float operands. |
| FR-369 CanonicalKeyUnavailable row | examined | spec/functional/FR-369-typed-checked-invariant-causes.md:44 | Collection ordering cannot obtain the canonical key of a member |
| QSpec FR-149-AC-26 | context_only | filament-specification spec/functional/type-model/FR-149-apply-complete-equality-matrix.md:446 | `set[+0, -0] = set[-0, +0]` is true |
| QSpec FR-144-AC-6, AC-25 | context_only | filament-specification spec/functional/type-model/FR-144-preserve-complete-collection-algebra.md:477,496 | a set, bag or ordered-set element type holding a float is admitted, keyed by the FR-148 `totalOrder` key |
