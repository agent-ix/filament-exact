---
id: SR-17940486709564847048055
title: "IR-1554 PR #186 FR-402 and TC-950 spec review"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #186 at c7a4541e; spec/functional/FR-402-kani-tractable-native-decimal-carrier.md (Behavior 11, Behavior 13, AC-13 to AC-15, Status), spec/test-cases/TC-950-kani-tractable-native-decimal-carrier.md (Description, steps 5 and 8-10, Verus record, Status), docs/verus-proofs.md; compared with filament-contract-codegen origin/main docs/qualification/ir1530-general-route-scalar-table.json (read-only)"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-402
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-950
    type: reviews
---

## Summary

Ticket: IR-1554. Spec review of the FR-402 and TC-950 edits.

- Behavior 11 matches what is proved. (a) is `lemma_normalized` and `row_normalized`, (b) is `lemma_admits_normalized` and (c) is `row_from_bindings`. The domain is every `i128` coefficient, every `i64` binding, every `u32` scale and every native type, with no precondition. The "does not cover" list (the checked type constructor, the ingress, the read-out) matches the `external` and unannotated functions. The library trusted base is named: `checked_mul`, `u32::try_from`, `i128::from`, and i128 `/` and `%`. The #181 FND-003 pattern does not recur.
- The POWERS_OF_TEN assumption is stated in Behavior 11, in FR-402 Status, in TC-950 (Verus record and step 10) and in the docs/verus-proofs.md trust table, and it has its own AC (AC-15).
- AC-13, AC-14 and AC-15 are atomic and tagged in the matrix. The Verus record gives counts and wall times only, with no tool version or pin. Cargo.toml, Cargo.lock and KERNEL_VERSION are unchanged.
- FR-402 Status names what stays unproved (the type constructor, the ingress, the read-out, and agreement with the owner's Decimal route) and names IR-1562 for the CG half. IR-1562's 2026-11-17 scope comment adds this decimal row. The #175 and #181 patterns (Status overclaiming, a deferral without an owned ticket) do not recur.
- The CG table on origin/main has 74 rows and none for decimal. No row claims more than this proof gives (FND-002).
- TC-950 still carries the pre-proof text (FND-001). This repeats #181 FND-002.

## Verdict

CHANGES REQUIRED (one medium, doc-only). Two TC-950 sentences written before this PR still say AC-13 is not executable and has not run. They contradict steps 8 to 10, the Verus record, and the paragraph this PR adds to the same Status section. One low: the FR-402 Status says the CG table entry is "deferred to IR-1562" without saying that the table has no decimal row yet.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-950 contradicts itself about AC-13. Step 5 still ends "AC-13 is not executable: it waits on a separate Verus track." The Status paragraph still ends "No Kani harness remains. AC-13 and step 7 (AC-11) have not run." This PR adds step 8 (AC-13 runs `make verus`), the Verus record ("440 verified, 0 errors") and the Status line "Steps 8 to 10 (AC-13 to AC-15, IR-1554): the Verus record above". The stale text is the #181 FND-002 pattern again. Delete the AC-13 sentence from step 5, and change the Status sentence to "Step 7 (AC-11) has not run." | spec/test-cases/TC-950-kani-tractable-native-decimal-carrier.md:34; spec/test-cases/TC-950-kani-tractable-native-decimal-carrier.md:105 |
| FND-002 | low | FR-402 Status says "Recording the Verus verdict in the CG tractability table is a CG change, deferred to IR-1562". The CG table (filament-contract-codegen origin/main, docs/qualification/ir1530-general-route-scalar-table.json, 74 rows) has no decimal row: no `not_tractable` row to replace, and no native decimal row, since CG's decimal row waits on IR-259. The sentence reads as if a row exists to update. Say that IR-1562 adds the decimal row, with its scope and the POWERS_OF_TEN assumption, when the table records it. | spec/functional/FR-402-kani-tractable-native-decimal-carrier.md:84 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-402 Behavior 11 | examined | A Verus proof SHALL prove the following on the production functions themselves, for every signed 128-bit coefficient, every `i64` coefficient and scale binding, every `u32` scale and every native type, with no bound. ... It assumes and does not prove the prover library's specifications of `i128::checked_mul`, `u32::try_from` on an `i64`, `i128::from` on an `i64`, and the truncating division and remainder of `i128`, and one fact about this crate: that the table `POWERS_OF_TEN` holds `10^0` through `10^38`. |
| FR-402 Behavior 13 | examined | The kernel SHALL claim a full-width proof of the constructor and the normalized view only as Behavior 11 states it, with its assumptions. |
| FR-402-AC-13 | examined | The Verus proof of Behavior 11 verifies with no error (`make verus`) on the production functions Behavior 11 lists. |
| FR-402-AC-14 | examined | Every mutant of `scripts/ir1554_verus_mutants.py` makes the proof of Behavior 11 fail with a failed proof obligation ... |
| FR-402-AC-15 | examined | The table `POWERS_OF_TEN` ... holds `10^0` through `10^38`: a host test compares every entry with a power computed by repeated checked multiplication, and checks that `10^39` leaves signed 128 bits. |
| FR-402 Status | examined | The full-width proof of the constructor and the normalized view is delivered in Verus (IR-1554 ...), with the power table as its one assumption about this crate. What stays unproved: ... Recording the Verus verdict in the CG tractability table is a CG change, deferred to IR-1562 ... |
| TC-950 Description, steps 5 and 8-10, Verus record, Status | examined | Verify FR-402-AC-1 through FR-402-AC-15 ...; 440 verified, 0 errors; 12 of 12 mutants killed ... |
| docs/verus-proofs.md | examined | Two files and one constant hold everything the proofs take on trust. ... POWERS_OF_TEN in src/native_decimal.rs |
| CG ir1530-general-route-scalar-table.json | context_only | 74 rows; no decimal row |
| IR-1562 | context_only | Scope addition: the decimal constructor/normalized-view row (IR-1554, FR-402-AC-13) joins this batched CG table PR. |
