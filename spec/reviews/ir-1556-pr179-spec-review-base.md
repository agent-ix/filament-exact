---
id: SR-17940486709564847048049
title: "IR-1556 PR #179 FR-409 and TC-957 spec review"
type: SpecReview
analysis: base
date: 2026-11-17
scope: "agent-ix/filament-exact PR #179 at 56d9d8b0; spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md (Behavior 24, AC-11, AC-12, Status), spec/test-cases/TC-957-native-integer-divide-modulo-plan.md (Description, steps 10-11, Verus record, Status)"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-957
    type: reviews
---

## Summary

Ticket: IR-1556. Spec review of the FR-409 and TC-957 edits.

- Behavior 24 and the proof agree. The windows of `in_window` are Behavior 15's windows: truncating `[0, |b|-1]` for `a >= 0` and `[-(|b|-1), 0]` otherwise; floor `[0, b-1]` for `b > 0` and `[b+1, 0]` otherwise; Euclidean `[0, |b|-1]`. The behaviour says the quotient is absent "only for" `MIN / -1`; the ensures states it as an equivalence, which is stronger, so they do not conflict.
- `Modulo` is the Euclidean remainder of the same function: `evaluate_native_integer_modulo` passes `DivisionProfile::Euclidean`.
- The zero divisor is a precondition of the proof. The `Undefined(DivisionByZero)` refusal is kept by the caller and is stated as out of scope.
- The FR Status and the TC Status claim the full-width relation for `native_div_rem` only. The 20 timeout rows stay measured, so the old Behavior 19 claims are not overturned and nothing is overclaimed.
- No pin, tool version or "pinned release" appears in the diff. The Verus record gives only a run count and wall times, so #175 FND-002 is not repeated.
- I found no conflict between Behavior 24 and AC-11 or AC-12, so #175 spec FND-001 is not repeated.
- AC-12 is atomic. AC-11 is not (FND-001).
- The list of assumptions in Behavior 24 is incomplete (FND-002).

## Verdict

The statement is accurate and honest about its scope. Two low findings: one compound criterion, and an incomplete trusted-base list in Behavior 24.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-409-AC-11 is compound. It bundles three checks with different failure modes: the Verus run verifies; the run is recorded in TC-957 (an inspection); and the timeout rows of Behavior 19 stay measured and claim no Kani proof (an inspection AC-10 already owns). This is the compound-AC pattern of #175 spec FND-002. Keep AC-11 to "verifies with no error, recorded in TC-957" and drop the timeout-row clause, or move it to AC-10. | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:146 |
| FND-002 | low | Behavior 24 says the proof "assumes the prover library's specification of `checked_div`, `wrapping_add` and `wrapping_sub`". `native_div_rem` also relies on the library specifications of `checked_sub` and `checked_add`, which are the quotient steps that the proof shows never overflow. Name them, or say "the library specifications of the checked and wrapping `i128` operations it calls". | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:137 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-409 Behavior 24 | examined | A Verus proof SHALL prove, on the production function `native_div_rem` itself and for every `i128` dividend and every nonzero `i128` divisor under each of the three profiles, that the returned remainder lies in the profile's window of Behavior 15, that a returned quotient `q` satisfies `a = q b + r` exactly, and that the quotient is absent only for `i128::MIN / -1`, whose remainder is zero. ... |
| FR-409 Behavior 15 windows | context_only | truncating `[0, \|b\| - 1]` when `a >= 0` and `[-(\|b\| - 1), 0]` otherwise; floor `[0, b - 1]` when `b > 0` and `[b + 1, 0]` otherwise; Euclidean `[0, \|b\| - 1]` |
| FR-409-AC-11 | examined | The Verus proof of Behavior 24 verifies with no error (`make verus`) on the production function `native_div_rem`. The run is recorded in TC-957's evidence record. The timeout rows of Behavior 19 stay recorded as measured and claim no Kani proof. |
| FR-409-AC-12 | examined | Every mutant of `scripts/ir1556_verus_mutants.py` makes the proof of Behavior 24 fail with a failed proof obligation (a postcondition, precondition or assertion that does not hold, or a possible overflow). A run that only exhausts the solver's resource limit is no verdict and does not count. |
| FR-409 Status paragraph (IR-1556) | examined | The full-width relation is proved by Verus ... the compiled behaviour is the same and `KERNEL_VERSION` is unchanged. The 20 timeout rows stay as measured rows under Kani. |
| TC-957 steps 10-11, Verus record, Status | examined | `make verus` 221 verified, 0 errors, 4.6 s wall; 10 of 10 mutants killed by a failed postcondition, none UNRELIABLE |
