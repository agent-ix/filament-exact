---
id: SR-17940486709564847047998
title: "IR-1541 PR #171 FR-409 / TC-957 evidence and Status review"
type: SpecReview
analysis: base
date: 2026-11-15
scope: "agent-ix/filament-exact PR #171 at 4002545d, git diff origin/main...HEAD; IR-1541; spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md (Status), spec/test-cases/TC-957-native-integer-divide-modulo-plan.md (Evidence record, Status)"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-957
    type: reviews
---

## Summary

Ticket: IR-1541. The spec change fills TC-957's evidence record (80 rows) and replaces the PLANNED Status of FR-409 and TC-957 with a run summary.

Faithfulness check, scripted, of every evidence row against the coder's raw `ir1541-results.tsv` (80 rows) and `ir1541-mutants2.tsv` (53 rows):

- Outcome: all 80 match (42 SUCCESSFUL to VERIFIED, 19 TIMEOUT to `timeout (600 s)`, 19 `not run, covered by <smallest failing width>`). Each `not run` names the right covering row, and the ladder order is ascending.
- Time, peak and covers: every executed row's wall seconds and `memory.peak` GiB equal the tsv. The covers cell equals the tsv cell, and the required covers are `ok` on every VERIFIED row.
- Timeout rows are labelled as measured rows (`timeout (600 s)`, covers `not reached`, no mutant), and the Status says no proof is claimed for them.
- The assigned-mutant cell equals the mutant tsv for all 41 VERIFIED rows that have a run. `modulo_const_neg1_w128` says `not run`.
- Counts in Status: 42 + 19 + 19 = 80, and 30 REPLAY-DIFFERS + 23 REPLAY-SAME = 53. These are correct as counts of the tsv labels. Peaks are at most 2.28 GiB.

The tsv labels themselves are wrong for the replay column (FND-001; mechanism in the code-review artifact SR-17940486709564847047996, FND-001).

## Verdict

Not mergeable as is. The measured-row record is faithful to the raw results, and timeout rows are honest measured rows. The AC-9 replay status is not honest in its cause. The Status says the 23 REPLAY-SAME rows fail because 'the playback decoder does not reproduce the distinguishing input'. In fact the decoded inputs do distinguish, and the recorded mutant outcomes differ from `divide`/`modulo` over `Integer` in all 23. The tooling compared against a stale binary. After the replays are re-run, AC-9 is likely met for all 53. `modulo_const_neg1_w128` (FND-002) and the probe record (FND-003) are cheap to close in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-409 and TC-957 Status say 23 Kani mutant playbacks are REPLAY-SAME because 'the playback decoder does not reproduce the distinguishing input', so FR-409-AC-9 is PARTIAL for them, and TC-957 labels those rows REPLAY-SAME. Both statements are false. The decoded inputs are distinguishing: for example `-20 div 8` under domain `[-2,-1]` gives truncating `-2` in the domain and floor `-3` outside it. The recorded mutant outcome differs from `divide`/`modulo` over `Integer` in all 23 (21 value or outcome differences, 2 panics). The 'healthy' side was a stale previous-mutant binary. Re-run on the healthy crate, healthy plan == exact for all 50 evaluation playbacks. Fix the replay tooling, re-run the 53 replays, relabel the evidence record, and restate AC-9's status from the new run. Until then the record carries replay verdicts that a broken tool produced. | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:156, spec/test-cases/TC-957-native-integer-divide-modulo-plan.md:63-141 |
| FND-002 | medium | `modulo_const_neg1_w128` is recorded as VERIFIED, but its assigned mutant (result interval skipped) never ran. Behavior 19's VERIFIED state requires 'its assigned harness mutant ... the harness fails', so the row is in none of the three settlement states. The Status discloses it ('its mutant is PARTIAL'), but the Outcome cell still says VERIFIED. The healthy run took 47 s, so the mutant run is cheap. Run it, or record the outcome as not settled rather than VERIFIED. | spec/test-cases/TC-957-native-integer-divide-modulo-plan.md:142, spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:156 |
| FND-003 | medium | The Behavior 17 probe records are missing from TC-957 and the Status. FR-409-AC-7 requires 'The probe of Behavior 17 ran before the full set, and its two records exist with time and peak'. The coder's `ir1541-probe.final.tsv` holds them: `trunc_quotient_sym_w8` SUCCESSFUL 47.8 s 1.67 GiB, and `trunc_quotient_sym_w64` TIMEOUT 600 s 1.67 GiB. The evidence record lists `trunc_quotient_sym_w64` as 'not run, covered by trunc_quotient_sym_w32', although the probe ran it to a 600 s timeout. Add the two probe records with their provenance, and say in the w64 row that the probe measured it. | spec/test-cases/TC-957-native-integer-divide-modulo-plan.md:57-69 |
| FND-004 | low | The Status says '42 are VERIFIED with every cover reached', while the Covers column shows `3/6` and `2/6`. The unreachable covers belong to other member combinations compiled into the shared harness body, and the required covers are all reached. State that the column counts all compiled covers and that every required cover of the shape was reached, or print required covers only. | spec/test-cases/TC-957-native-integer-divide-modulo-plan.md:57, spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:156 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-409-AC-7 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:141 | The probe of Behavior 17 ran before the full set, and its two records exist with time and peak. Every harness of AC-5 has one terminal record in one of Behavior 19's three states ... |
| FR-409-AC-8 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:142 | Each measured row appears in TC-957's evidence record, and the kernel holds no other copy of it. The row names the operation kind, profile, member, divisor shape and smallest failing width ... |
| FR-409-AC-9 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:143 | Each mutant of Behavior 22 fails the host differential at its distinguishing input ... An evaluation-harness playback replayed through `divide` or `modulo` over `Integer` gives a different value or outcome from the mutant's ... |
| FR-409-AC-10 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:144 | No proof is claimed for a harness without an executed VERIFIED verdict. The Status records, per member combination, shape and width, the verdict or the table row, and claims none before the runs exist. |
| FR-409 Status | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:156 | Run under IR-1541. ... so the replay requirement of FR-409-AC-9 is PARTIAL for those 23. `modulo_const_neg1_w128` has no mutant run, so its mutant is PARTIAL. |
| TC-957 Evidence record | examined | spec/test-cases/TC-957-native-integer-divide-modulo-plan.md:57-142 | Every harness ran once under `cargo kani` with a 600-second wall-clock limit ... |

## Dispositions

Disposition pass 1, against fix commit 7eac59d1. Scripted re-check of TC-957 against `ir1541-results.tsv`, `ir1541-probe.final.tsv` and `ir1541-mutants3.tsv`: 80 ladder rows, 42 VERIFIED / 20 timeout / 18 not run. Every VERIFIED time, peak and assigned-mutant cell matches the tsv, and every not-run row matches its covering row. The `trunc_quotient_sym_w64` timeout row equals the probe record (600.0 s, 1.67 GiB). The FR-409 and TC-957 Status counts match.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Status now reads '54 of 54 REPLAY-DIFFERS' and names the earlier cause as the shared build directory, not the decoder. TC-957 rows carry REPLAY-DIFFERS from the recomputed tsv, and the reviewer's independent healthy replay agrees. |
| FND-002 | fixed | `modulo_const_neg1_w128` now records `result-interval-skipped FAILED REPLAY-DIFFERS`. The log `mutant-result-interval-skipped-modulo_const_neg1_w128.log` has VERIFICATION:- FAILED, 122.6 s, memory.peak 14335586304 B (13.35 GiB, under 16 GiB). The reviewer reproduced the healthy replay (plan == exact, ModuloOutOfDomain), and the mutant completes 0. |
| FND-003 | fixed | A probe table now records both runs: `probe trunc_quotient_sym_w8` VERIFIED 47.8 s 1.67 GiB, and `probe trunc_quotient_sym_w64` timeout 600.0 s 1.67 GiB. These equal `ir1541-probe.final.tsv`. The `trunc_quotient_sym_w64` row is now the probe's measured timeout rather than 'not run', and Status says so. |
| FND-004 | fixed | The Status now says 'every cover that belongs to the harness reached', and the column is 'Required covers reached' with an explanation of Kani's total. |
