---
id: SR-17940486709564847047965
title: "Code review — filament-exact Undefined::MalformedNumber"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-exact; src/outcome.rs; spec/functional/FR-414-undefined-malformed-number.md; spec/test-cases/TC-962-undefined-malformed-number.md"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-exact/FR-414", type: references }
---

## Summary

Reviewed PR #190 at head `a3202ec4` against `b25ff96f`. The new public variant and TC-962 behavior satisfy FR-414; one low-severity trace-diagnostic finding remains.

## Verdict

**CONDITIONAL** — the test binds FR-414-AC-1, but its prose repeats the criterion ID outside the trace marker and adds a Quire diagnostic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-962 doc comment repeats `FR-414-AC-1` outside `#[trace]`; `quire coverage` reports a new `trace-id-outside-marker` diagnostic for that prose even though the test is correctly bound. Remove the criterion ID from the prose and leave the binding in the attribute. | src/outcome.rs:470-472 |

## Review Notes

The enum variant is public, unit-shaped, documented with the FSpec spelling and text-to-number condition, and inherits the existing `PartialEq`/`Hash` derives. The test compares the variant with every existing undefined reason, verifies clone equality, and verifies that the outcome is `Undefined` with no completed value. FR-414-AC-2 and AC-3 use Inspection and correctly appear as `method-without-symbol` in the computed matrix. The evaluator's text conversion paths are in the consuming crate and already use the new typed variant; this PR supplies that kernel API.

No applicable AssuranceProfile was found. No spec, dependency, CI workflow, or feature gate changed.

## Gates

- `cargo test -p filament-exact tc_962_malformed_number_is_a_distinct_undefined_reason`: 1 passed.
- `cargo clippy -p filament-exact --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed; stable rustfmt emitted the repo's existing warnings that `imports_granularity` and `group_imports` require nightly.
- `cargo deny check`: passed; emitted existing unmatched license allowances and the wildcard `ix-trace-rs` branch dependency warning.
- `git diff origin/main...HEAD --check`: passed.

Full crate tests were not run; the lane's current plan calls for focused tests.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Removed the criterion ID from the prose; TC-962 remains in the trace tag. |

After excerpt: `/// TC-962: malformed text-to-number input has its own undefined reason and is not a completed, refused or incomplete outcome.`
