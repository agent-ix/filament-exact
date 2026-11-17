---
id: SR-17940486709564847048046
title: "IR-1544 PR #175 FR-408 and TC-956 spec review"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #175 at e6624e2a; IR-1544; spec/functional/FR-408-kani-tractable-native-rational-carrier.md (Behavior 18, 18a, AC-11, AC-13, Limitations, Status), spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md (Description, step 12, evidence and Verus records, Status)"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-956
    type: reviews
---

## Summary

Ticket: IR-1544. This reviews the FR-408 and TC-956 edits of PR #175. I compared Behavior 18a's claims with what the Verus theorems state, which matches: (a) from_i64_parts; (b) add, subtract and multiply never refuse and complete to lowest terms after the Behavior 11 requests, ordering compares a q with c p after the Behavior 12 requests, incomplete is a proper prefix; (c) same pair iff a q = c p; plus reduce, gcd8, gcd64, gcd256, bits and the capacity checks over all i128. The Limitations paragraph and the TC-956 note ("they do not state that a sufficient Meter admits them") are honest about the Meter model.

## Verdict

Behavior 18a matches the proofs, and nothing is overclaimed beyond what code-review FND-001 and FND-002 cover. One consistency gap: Behavior 18 still requires a Kani proof that AC-11 now says is not given. One low compound-AC finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Behavior 18 was not amended. It still says "A Kani harness SHALL prove ... (a) `from_i64_parts` over all symbolic `i64` pairs with no unwind bound". The rewritten FR-408-AC-11 says Behavior 18 (a) "claim[s] no Kani proof; their executed verdict is the Verus proof of Behavior 18a". So the statement still requires what its criterion now waives, and AC-11 is "executed" only because its text was changed to fit the evidence. Amend Behavior 18 so that (a) and the full-i64 rows of (b) are measured rows under Kani, with the SHALL carried by 18a. | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:93, spec/functional/FR-408-kani-tractable-native-rational-carrier.md:116 |
| FND-002 | low | FR-408-AC-13 is compound. It bundles the Verus run, the trusted-base boundary, the Verus mutant set, and a separate Kani harness with its own mutant, so one partial failure cannot be reported against one criterion. Split the Kani equality harness into its own AC, or attach it to AC-11. | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:118 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-408 Behavior 18 | examined | A Kani harness SHALL prove ... (a) `from_i64_parts` over all symbolic `i64` pairs with no unwind bound; (b) ... over symbolic parts drawn from the full `i64` domain where the measurement finishes ... |
| FR-408 Behavior 18a | examined | A Verus proof SHALL prove the following on the production functions themselves, for every input of the stated domain and with no bound. (a) ... (b) ... (c) Two such carriers are the same pair exactly when `a q = c p`. ... Each proof SHALL fail on a mutant of the operation it covers. |
| FR-408-AC-11 | examined | The Kani harnesses of Behavior 18 (b over the CG domain, c) return an executed terminal VERIFIED verdict ... Behavior 18 (a) and the full-`i64` rows of Behavior 18 (b) are measured not-tractable rows under Kani and claim no Kani proof ... |
| FR-408-AC-13 | examined | The Verus proofs of Behavior 18a verify with no error under the pinned Verus release (`make verus`) ... and the Kani harness `native_rational_equal_all_parts_is_structural` is VERIFIED with its covers reached and fails its mutant. |
| FR-408 Limitations paragraph | examined | The Verus proof of Behavior 18a is unbounded over signed 64-bit parts for the six rows, and over signed 128 bits for the reduction, the gcds and the capacity checks; it models the `Meter` only as far as Behavior 18a states. |
| FR-408 Status | examined | ... AC-11 and AC-13 are executed. For the prover, `form` names the refusal constructor through an alias instead of binding it to a local; no other compiled line of a production body changed, and `KERNEL_VERSION` is unchanged. |
| TC-956 Verus record | examined | `make verus` 208 verified, 0 errors; `make verus-mutants` 17 of 17 mutants fail; row-to-theorem table |
| TC-956 Status | examined | AC-11 is executed: b and c are VERIFIED under Kani, and (a) with the other five full-`i64` rows ... has the Verus record above as its executed verdict |

## Dispositions

Round 1, fix head ea7e1614 (reviewed head e6624e2a). origin/main has moved to 234f233 since the review; `git merge-tree` against it is clean. Reviewer re-runs in a fresh private worktree under flock /tmp/cargo-slot.lock: make verus 211 verified, 0 errors (6.0 s wall, 0.55 GiB); make verus-mutants: unmutated copy verified, 18 of 18 killed, every one by `postcondition not satisfied`, 58 s wall. Probe: the PR's mutant script, run on a whitespace-only mutant under `--rlimit 0.05` so that only resource-limit errors appear, reports `UNRELIABLE` and exits 1. Reviewer re-mutation after the opaque/reveal restructuring: the assert(false) vacuity probes in row_add, row_order and row_from_i64_parts each fail; the mutated ensures (add denominator, swapped order cross products, equality lemma, canonical with gcd<=2) and code (denominator<0, bits max(0), steps8_16 cut to three quarters, ordering charge point) each fail a postcondition, so no ensures was weakened. Observation, not a finding: a code-only edit of an unrolled ladder (steps_128 or gcd_steps_256 cut to one half, spec unchanged) still ends only in rlimit. That is not a pass, so soundness holds, and the recorded mutants of those ladders edit code and spec together and fail the theorem. Gates at ea7e1614: cargo fmt --check clean; make lint and clippy --locked --all-targets -D warnings pass; make test 88 result lines, 0 failed; build-no-std, docs and deny (advisories ok, bans ok, licenses ok, sources ok) pass; git status clean. The committed spec/reviews/ir-1544-pr175-*.md copies were byte-identical (cmp) to the reviewer's round-0 files.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: 18. A Kani harness SHALL prove ...: (b) ... over symbolic `i64` parts constrained to the CG row's domain ...; (c) the composed predicate ... The harnesses of (a), `from_i64_parts` over all symbolic `i64` pairs with no unwind bound, and of the operations of (b) over symbolic parts drawn from the full `i64` domain SHALL be kept and measured. They are measured rows under Kani and are not required to verify; the requirement to prove (a) and the full-`i64` rows is Behavior 18a. |
| FND-002 | fixed | after: FR-408-AC-13: The Verus proofs of Behavior 18a verify with no error (`make verus`) ... FR-408-AC-14: Every mutant of `scripts/ir1544_verus_mutants.py` makes a Verus proof of Behavior 18a fail with a failed proof obligation ... FR-408-AC-15: The Kani harness `native_rational_equal_all_parts_is_structural` is VERIFIED with its covers reached ... and fails its mutant. |
