---
id: SR-17940486709564847048058
title: "IR-1555 PR #188 FR-403 and TC-951 spec review"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #188 at 4e54d122; spec/functional/FR-403-native-decimal-operations-refine-owner-path.md (Behaviors 3, 41, 42, AC-15 to AC-17, Status, Dependencies), spec/test-cases/TC-951-native-decimal-operations-match-owner-path.md (Description, steps 9 and 12-14, Verus record, Status), docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-403
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-951
    type: reviews
---

## Summary

Ticket: IR-1555. Spec review of the FR-403 and TC-951 edits.

- **Disclosure (a) holds.** Behavior 41 says: "The statement proved is the mathematical one above. It is not equality with `evaluate_decimal` and `order_numbers`: the owner route is not modelled in the prover, and the agreement of the native path with it rests on the host evidence of Behaviors 35 to 38". The grounds are proved "for the native plan", not the owner's plan of Behavior 3, and Behavior 41 says that too.
  - FR-403 Status lists the owner agreement under "What stays unproved". So do TC-951 ("Not proved") and the planner ruling on IR-1555.
  - docs/verus-proofs.md says only "the native Decimal operations and ordering over the full width".
  - The old "full-width refinement proof" wording is gone from Behavior 41/42 and AC-15.
  - No text claims equivalence with the executable owner route.
  - The CG sentence says the CG table has no decimal row and that IR-1562 adds it. That matches the earlier lane's finding, and the #181 CG-row pattern does not recur.
- **Behavior 41 matches what is proved.** (a) is the planner postconditions plus `admitted_is`. (b) is `tail_means` via `lemma_admitted`, with `rounds` and `lemma_rounds_unique`. (c) is `loss_is`. (d) is `settled_is`. (e) is `value_order` and the exact `widening_mul`. (f) is `bits_of` and `digits_of`. The "does not cover" list (the charge amounts, the charge sequence, the Meter, `OrderingOperator::holds`, the read-out) matches the functions that have only overflow postconditions or no spec. The domain is every i128 coefficient, every u32 scale, every type and every mode. The entry points have no precondition.
- **The assumed library specs are complete.** The prover-library list and the `src/verus_std_specs.rs` list in Behavior 41 cover every library call in the verified bodies. They are the same as the docs trust table. The #181/#186 incomplete-list pattern does not recur.
- **AC-15 and AC-16 are atomic and matrix-tagged.** The Verus record gives counts and wall times only, with no tool version or pin. The excluded rlimit-only candidate (`10^39`) is recorded rather than counted. Cargo.toml, Cargo.lock and KERNEL_VERSION are unchanged.
- The Status no longer says "No proof result or measurement exists"; it now says "no CG measurement exists". This is correct. No stale pre-proof text remains.

## Verdict

Two low findings. The proof scope is stated honestly and nothing overclaims.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-951 Description says FR-403-AC-1 through AC-17 are verified "by running the native operations beside the owner's `evaluate_decimal` and `order_numbers`". AC-15 and AC-16 are Verus runs and AC-17 is an inspection (steps 12 to 14). The sentence contradicts the TC's own Verus record. | spec/test-cases/TC-951-native-decimal-operations-match-owner-path.md:14 |
| FND-002 | low | FR-403-AC-17 joins two independently checkable claims: no Kani proof is claimed, and the Status states what the Verus proof covers and does not. One can hold while the other fails. This repeats the compound-AC pattern fixed in the earlier lanes. | spec/functional/FR-403-native-decimal-operations-refine-owner-path.md:122 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-403 Behavior 41 | examined | A Verus proof SHALL prove the following on the production functions themselves, for every pair of retained operands ... The statement proved is the mathematical one above. It is not equality with `evaluate_decimal` and `order_numbers` ... Each proof SHALL fail on a mutant of the function it covers. |
| FR-403 Behavior 42 | examined | The kernel SHALL claim no Kani proof of the six operations or the ordering, and SHALL claim a full-width proof of them only as Behavior 41 states it, with its assumptions and exclusions. |
| FR-403-AC-15 | examined | The Verus proofs of Behavior 41 verify with no error (`make verus`) on the production functions Behavior 41 lists. |
| FR-403-AC-16 | examined | Every mutant of `scripts/ir1555_verus_mutants.py` makes a proof of Behavior 41 fail with a failed proof obligation ... |
| FR-403-AC-17 | examined | No Kani proof of the six operations or the ordering is claimed, and the Status states what the Verus proof covers and what it does not (Behavior 42). |
| FR-403 Status | examined | The full-width proof is delivered in Verus (IR-1555, Behavior 41, AC-15 and AC-16 ...). What stays unproved: the agreement with the owner route ... The CG tractability table has no decimal row yet. IR-1562 adds the decimal operation and ordering rows ... |
| TC-951 Description | examined | Verify FR-403-AC-1 through FR-403-AC-17 by running the native operations beside the owner's `evaluate_decimal` and `order_numbers` ... |
| TC-951 Verus record | examined | `make verus` 630 verified, 0 errors; 53 of 53 mutants killed by a failed proof obligation, none UNRELIABLE; claims (a)-(f) table; mutant table; 10^39 candidate excluded |
| docs/verus-proofs.md | examined | The fifth is the native Decimal operations and ordering over the full width (FR-403 Behavior 41, TC-951). Trust table row for src/verus_std_specs.rs. |
| FR-403 Behavior 3 | context_only | the kernel SHALL refuse admission exactly when at least one of the following holds for the owner's plan of the same operation, operands and type. In order: (a) ... |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, fix head bad06e13. The TC-951 Description now verifies AC-1 to AC-14 beside the owner route, AC-15 and AC-16 by the Verus runs (steps 12 and 13), and AC-17 and AC-18 by inspection (steps 14 and 15). |
| FND-002 | fixed | Round 1, fix head bad06e13. AC-17 is split into AC-17 (no Kani proof claimed) and AC-18 (the Status states coverage). TC-951 steps 9, 14 and 15 and its Status match the split. `quoin matrix` lists AC-17 and AC-18 as Inspection (`method-without-symbol`). No stale step or AC reference remains. |
