---
id: SR-16706
title: "Gap analysis of filament-exact PR #195: clock-place reference kernel value"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-exact@f7ced1d7aa8965ed8ca001ed6114d28d6d7d0428; PR #195 diff: FR-415-AC-1 to FR-415-AC-6, TC-963 and their tagged tests in src/clock_reference.rs"
review_set: subset
---
# Gap analysis of filament-exact PR #195

## Summary

Ticket: QSL-2776 (step 1). Plan completion: not assessed. `quoin matrix
--json` (quoin 0.36.8, run locally on a detached checkout of f7ced1d7) lists
FR-415-AC-1 to AC-6 each with tagged tests: AC-1 two, AC-2 six, AC-3 one,
AC-4 two, AC-5 two, AC-6 one. Every new line of production code is owned by
FR-415; there are no stubs and no untraced code. The 15 tests pass.

Bindings checked against TC-963:

- AC-1: type admission, the three wrappers and the clock-keyed map type are
  exercised; Text is not (FND-002); map entry order is the known open spec
  conflict (FND-005).
- AC-2: both accepted texts and all 19 refused texts of TC-963 step 2 are
  present, plus extra vectors (empty text, `"07"` index, duplicate slot,
  numeric index). The non-UTF-8 vector is on the wrong component (FND-001).
- AC-3: both accepted texts round trip, equality `true`, key `Equal`.
- AC-4: all equal/unequal/foreign vectors with plan equal to the Boolean-leaf
  plan; mixed-kind with object reference and String but not Text (FND-003).
- AC-5: all key vectors, universe first, `None` for the mixed pair, set order
  for three insertion orders.
- AC-6: `occ` one; the size assertion cannot fail (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-963 step 2 refuses "A with its second component ff" (K). The test sets `components[2]`, which is P (index 0 and 1 are `place` and `v2`). Rule 3 is still exercised, but not on the vector the TC names. | src/clock_reference.rs:389-390 |
| FND-002 | low | FR-415-AC-1 requires `Text` to reject a clock-place reference and `ValueType::ClockReference` to reject a Text value holding the place id text. Neither direction is tested; String, Bytes, JsonObject and object Reference are. | src/clock_reference.rs:273-294 |
| FND-003 | low | FR-415-AC-4 requires a direct plan pairing a clock-place reference with a Text to return `ValueKindMismatch`. The test covers object Reference and String only. | src/clock_reference.rs:505-508 |
| FND-004 | low | FR-415-AC-6 "accounts 279 bytes" is tested as `reference.text().len() == 279`, which only restates the fixture length (already asserted in `the_fixture_builder_reproduces_the_fsl_literal`). The kernel has no variable-size function for any kind; callers charge through `Charge::size`. FR-415 Behavior 8 and FR-385 Behavior 6 should say the caller charges `text().len()`, or the kernel should expose a size function the test can pin. | src/clock_reference.rs:583-590 |
| FND-005 | medium | Known open, routed to arch, not counted against this PR. FR-415-AC-1 says the `Map<ClockReference, Integer>` built from `A(K="c/e")` then `A(K="c/d")` holds `"c/d"` first; FR-410 Behavior 1 and FR-410-AC-1 say `entries()` keeps the supplied order, and `from_admitted_map` stores entries unsorted. The test asserts only `entries().len() == 2`. | src/clock_reference.rs:325-337 |

## Verdict

Coverage is complete except for three small vector gaps (FND-001 to FND-003)
and a size assertion that cannot fail (FND-004, a spec-wording question).
FND-005 is a real spec conflict between FR-415-AC-1 and FR-410 that needs an
arch ruling; it does not block this PR.
