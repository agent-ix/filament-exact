---
id: SR-17940486709564847047966
title: "Gap analysis — filament-exact repository audit"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-exact; spec/, src/, tests/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-exact/FR-414", type: references }
---

## Summary

Planless repository audit of filament-exact. The changed FR-414-AC-1 is tagged by TC-962, and the PR improves whole-repo static coverage by one criterion; 127 untagged and 10 ignored-only criteria remain from the existing repository baseline.

## Verdict

**FAIL** — the repository-wide matrix still contains untagged criteria and criteria backed only by ignored tests; this PR introduces neither gap.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed matrix has 127 untagged criteria. The base at `b25ff96f` had 128; FR-414-AC-1 is now tagged by TC-962, so the remaining set is pre-existing. | quoin matrix: 127 untagged criteria |
| FND-002 | high | Ten criteria are tagged only by ignored tests and therefore have no running test evidence. This set is unchanged from base. | quoin matrix: 10 tagged-by-ignored-test criteria |

## Coverage

- Reconciliation: quoin matrix (quoin 0.36.7, quire 0.52.0); quire coverage inspected for untracked symbols and diagnostics.
- Plan completion: not assessed.
- Criteria after PR: tagged 278; untagged 127; tagged-by-ignored-test 10; method-without-symbol 50.
- Base at `b25ff96f`: tagged 277; untagged 128; tagged-by-ignored-test 10; method-without-symbol 50.
- Evidence: no run evidence for all 465 criteria; the changed AC is statically tagged, but the evidence store has no recorded runs.
- FR-414: AC-1 is tagged by `src/outcome.rs::tests::tc_962_malformed_number_is_a_distinct_undefined_reason`; AC-2 and AC-3 declare Inspection and correctly have `method-without-symbol`. No FR-414 test tag is untracked.
- Changed-surface reverse-gap inventory: one public enum variant, owned by FR-414; no unowned behavior or stub in the diff. A full symbol-by-symbol reverse-gap inventory of the repository was not completed.
- Semantic review: skipped (not opted in).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 127 remaining untagged criteria are inherited repository baseline gaps outside this single-ticket PR; base had 128, and this PR tags FR-414-AC-1. |
| FND-002 | deferred | The ten ignored-only criteria are unchanged from base and outside this single-ticket PR. |
