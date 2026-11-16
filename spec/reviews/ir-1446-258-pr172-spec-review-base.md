---
id: SR-17940486709564847048019
title: "IR-1446 / IR-258 PR #172 spec-review (base)"
type: SpecReview
analysis: base
date: 2026-11-15
scope: "agent-ix/filament-exact PR #172 at 5098e906, git diff origin/main...HEAD -- spec/; IR-1446, IR-258; spec/functional/FR-405-kani-tractable-native-float-carrier.md, spec/functional/FR-408-kani-tractable-native-rational-carrier.md, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md, spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-953
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-956
    type: reviews
---

## Summary

Tickets: IR-1446 (FR-405, TC-953), IR-258 (FR-408, TC-956). `quire validate` passes on all four files. No evidence document names a Kani version, and the Kani-version column and test are gone.

Numbers checked:

- TC-953 has 44 rows: 32 VERIFIED and 12 timeout. That is 7 NearestEven rows at 600 s and 5 Exact or directed rows at 1800 s, matching the FR-405 Status and TC-953 Status lists.
- The vacuity table has 32 rows, which matches the VERIFIED set.
- The two tool-fault rows name harnesses that exist.
- I spot-reran `float32_add_exact` (34.8 s against the recorded 35.3 s) and `native_fraction_gcd8_matches_euclid_over_u8` (9.0 s against the recorded 9.2 s).
- TC-956 has 28 rows: 22 VERIFIED and 6 full-i64 TIMED OUT at 900 s. Every VERIFIED row is under 600 s.

The FR-405 Behavior 21 amendment (10-minute probe, then a 30-minute cap) matches the lead's ruling. A probe timeout followed by an extended run is legitimate, not a goalposts move, provided the record says so. The record does not yet say so consistently (FND-002), and the amended SHALL was not applied to the seven NearestEven rows (FND-001). FR-408 Behavior 17b, AC-12 and TC-956 step 11 agree with the code. AC-11 stays PLANNED, which is honest.

## Verdict

Not mergeable as is. Four medium evidence-record findings must be fixed in this PR; all are text or measurement fixes, with no kernel change. Three low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Amended Behavior 21 says each harness 'SHALL be run with a 10-minute probe and, when it has not returned, again under a 30-minute cap'. TC-953's Limits paragraph says 'The seven NearestEven timeout rows were measured at the 600 s limit and were not extended'. Those seven rows did not get the cap run that the SHALL requires. Either rerun them under the 1800 s cap, or word Behavior 21 so that a probe timeout may stand as the measured row and the cap run is optional (stating who decides). | spec/functional/FR-405-kani-tractable-native-float-carrier.md:70, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:123 |
| FND-002 | medium | The record does not consistently say 'probe timed out, then extended', which the lead's ruling requires. The FusedMultiplyAdd Binary32 Exact row reads only '722.9 s (wall 724.7 s)'. The Limits paragraph lists 'The five Exact and directed rows that had not returned (five ids) and FusedMultiplyAdd Binary32 Exact', which does not state that this sixth row's 600 s probe timed out. The runner paragraph still says each row was 'wrapped in `timeout 600`', and step 8 still says 'a 10-minute wall-clock limit'. The five 1800 s timeout rows also do not record their 600 s probe. Fix: give each extended row's Time cell the form '600 s probe: timeout; 1800 s cap: VERIFIED 722.9 s (wall 724.7 s)' or '... 1800 s cap: timeout', and make step 8 and the runner paragraph state the probe and the cap. | spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:27, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:91, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:119, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:123 |
| FND-003 | medium | The AC-12 prose under the vacuity table is stale. It says 'Each of the 13 VERIFIED harnesses ... 1 for Negate, 2 for each comparison, 3 for each arithmetic harness ... The mutants are ... (`negate-quiets`, `numeric-equal-zeros`, `total-order-sign`, `bit-identical-numeric`, `tie-even-is-away`) ... All 13 failed their harness.' The table directly above it has 32 rows, 5 covers per fused harness, and `nan-payload-cleared` for 19 rows. Update it to 32 harnesses, the five-cover fused count and the full mutant list. | spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:166 |
| FND-004 | medium | TC-956 Status says that b (add, subtract, multiply, order, equality) and c are 'VERIFIED with their covers reached and mutants that fail them'. No Kani mutant was ever recorded for `native_rational_equal_cg_domain_matches_reference`: the earlier record had only multiply and order, and the new Mutants paragraph has add, subtract, multiply, the composed predicate and unreduced. The order mutant (113 s) ran on the pre-tier kernel. Behavior 18 requires each harness to fail on a mutant. AC-11 is PLANNED, so no credit is taken, but the planner's ruling sentence rests on this claim. Run an equality-row mutant and rerun the order mutant on the tiered kernel, or state exactly which rows have a mutant on this kernel. | spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:103, spec/functional/FR-408-kani-tractable-native-rational-carrier.md:92 |
| FND-005 | low | The deferral owners are not traceable. TC-956 and the FR-408 Status say '(owner: planner-filed Verus lane ticket)' and name no id. IR-1544 ('Verus: rational from_i64_parts gcd-equivalence over full i64 (IR-258 AC-11)') exists, and its AC-2 covers all six full-i64 rows, so name IR-1544. FR-405 Status and TC-953 say the float timeouts 'wait on a deductive prover (the Verus lane)', but no float Verus ticket was found in the IR team. File one or name the owner. | spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:93, spec/functional/FR-408-kani-tractable-native-rational-carrier.md:124, spec/functional/FR-405-kani-tractable-native-float-carrier.md:110 |
| FND-006 | low | TC-953 Status ends 'or, for the fused rows, on a model of `fma` that is correct for an infinite addend and a zero product'. That is stale: the fused harnesses now decide those regions by rule, and the remaining fused timeouts are solver hangs that wait on the deductive lane, as the Limits paragraph says. The Fused multiply-add paragraph's 'the kernel agrees with the rule in both regions on every path the solver explored' also implies a result for the timeout rows; limit it to the VERIFIED fused rows. | spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:125, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:185 |
| FND-007 | low | Behavior 17b sits between 17a and 17 (the order reads 17a, 17b, 17, 18), so the tier rule appears before the behaviour it extends. This is cosmetic, and 17a already set the pattern. Consider moving 17a and 17b after 17, or renumbering. | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:87-91 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-405 Behavior 19 (amended) | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:68 | A harness SHALL hold the outcome of the native operation in `ManuallyDrop` ... and SHALL decide by IEEE 754 rule any region where its host oracle's model is wrong, with the fault recorded in TC-953. |
| FR-405 Behavior 21 (amended) | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:70 | Each harness SHALL be run with a 10-minute probe and, when it has not returned, again under a 30-minute cap ... |
| FR-405-AC-11 | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:87 | ... recorded in TC-953's evidence record with its time, peak memory and harness id. |
| FR-405 Status | examined | spec/functional/FR-405-kani-tractable-native-float-carrier.md:95-111 | 32 of the 44 harnesses are VERIFIED ... |
| TC-953 Evidence record, Limits, Fused, Vacuity, Tool faults, Status | examined | spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:68-185 | evidence rows and prose |
| FR-408 Behavior 6, 17b | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:65,89 | 17b. The reduction SHALL take a third, narrowest tier ahead of the `u64` tier ... |
| FR-408-AC-10 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:112 | the reduction is the width-tiered reduction of Behaviors 17a and 17b ... |
| FR-408-AC-11 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:113 | PLANNED: ... The full-`i64` rows of Behavior 18 (b) are recorded as measured and are not required to verify. |
| FR-408-AC-12 | examined | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:114 | The `u8` tier of the reduction (Behavior 17b) agrees with Euclid's algorithm on every pair ... |
| TC-956 step 11, Evidence record, Mutants, Status | examined | spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:28-103 | 11. For AC-12, run the host sweep over every numerator in `[-600, 600]` ... |
| FR-408 Behavior 18 | context_only | spec/functional/FR-408-kani-tractable-native-rational-carrier.md:92 | Each harness SHALL fail on a mutant of the operation ... |

## Dispositions

Round 1, verified against fix commit cd71d77b.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Behavior 21 now reads: a 10-minute probe; a harness that has not returned MAY be run again under a 30-minute cap; the record SHALL state the last limit and whether the harness was extended. The seven NearestEven rows read '600 s (probe limit, not extended; ...)'. This matches the lead's probe/cap ruling. |
| FND-002 | fixed | Extended timeout rows read '1800 s (limit, after a 600 s probe that timed out; ...)'. Step 8 and the runner paragraph state the probe and the cap. The Limits paragraph says that FusedMultiplyAdd Binary32 `Exact`'s probe timed out and was extended (722.9 s), and that its final-text rerun (664.4 s) ran directly under the cap. |
| FND-003 | fixed | The AC-12 paragraph now gives 32 harnesses, 4 covers per arithmetic harness and 6 per fused harness, the full mutant list, and the two Exact rows with no verdict. |
| FND-004 | fixed | `equality-numerators-only` and `order-compares-a-p` were added to the script and recorded FAILED against the equality and order CG rows on the tiered kernel, in TC-956's new Kani mutant record. The test checks every pair. |
| FND-005 | fixed | TC-956 and the FR-408 Status name IR-1544 for the six full-i64 rows. FR-405 and TC-953 say 'no float Verus ticket exists yet; planner to assign'. |
| FND-006 | fixed | The TC-953 Status now says the unverified rows are solver hangs waiting on a deductive prover; the fma-model wording is gone. The Fused paragraph limits the claim that the kernel agrees with the rule to the three VERIFIED fused rows, and says nothing is claimed for the timeouts. |
| FND-007 | fixed | FR-408 now orders the behaviours 17, 17a, 17b, 18. |
