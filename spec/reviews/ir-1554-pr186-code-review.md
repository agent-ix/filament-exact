---
id: SR-17940486709564847048053
title: "IR-1554 PR #186 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #186 at c7a4541e (diff origin/main...HEAD, base 1f7c0d56, 9 files, +685/-8); IR-1554; src/native_decimal.rs, src/native_decimal/verus_proofs.rs, src/native_decimal/tests.rs, scripts/ir1554_verus_mutants.py, tests/verus_gate.rs, Makefile, docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-402
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-950
    type: reviews
---

## Summary

Ticket: IR-1554. Code review with the rust-review lane. The reviewer re-ran everything in a private worktree under `flock /tmp/cargo-slot.lock`, with logs in the worktree's `target/`.

- **Compiled code is unchanged.** Every line the diff adds to `src/native_decimal.rs` is a `cfg_attr(verus_keep_ghost, ...)` attribute, a `#[cfg(verus_keep_ghost)]` item or statement (the `vstd` import, the `verus!` block of closed spec accessors, the `proof!` call in `normalized_pair`, `mod verus_proofs`), or a host test in `tests.rs`. Cargo.toml, Cargo.lock, KERNEL_VERSION and src/json_object.rs are not touched.
- **Trusted base.** The new trust is `POWERS_OF_TEN`, marked `external_body` with `ensures powers_table(POWERS_OF_TEN)` (entries 0 to 38 equal `pow10`). The excluded functions are `NativeDecimalType::new` and `to_decimal` (`verus_verify(external)`); the `TryFrom<&DecimalType>` ingress is not annotated, so it is not verified. Both match Behavior 11's "does not cover". The PR adds no `assume`, `admit`, `axiom`, `assume_specification` or `opaque`. The library specifications used are those of `i128::checked_mul` (None exactly on overflow), `u32::try_from(i64)`, `i128::from(i64)` and i128 `/` and `%`. They are the real truncating `core` semantics, and Behavior 11 names them all. `strip_zeros` has one precondition, `power > 0`. Every caller passes `POWERS_OF_TEN[k]` and gets it from `lemma_powers_in_range`. The public theorems `row_from_bindings` and `row_normalized` have no precondition, so the domain is every `i64` coefficient and scale binding, every native type, and every retained `(i128, u32)` pair, including `i128::MIN` (`lemma_zeros_bound` allows magnitude 2^127).
- **Reproduced.** `make verus`: 440 verified, 0 errors, 8.3 s wall, 0.70 GiB peak. `make verus-mutants`: 6:17 wall, 0.72 GiB peak, sets 18/18, 10/10, 33/33 and 12/12 killed; no rlimit, UNRELIABLE, SURVIVED or INVALID.
- **What kills each mutant**, from full logs. The ten code-and-spec mutants fail a theorem: `lemma_strip_zeros` (two mutants; strip-ignores-the-scale also trips the `scale - places` underflow), `lemma_normalized` (three), `row_from_bindings` (membership-skips-normalization) and `lemma_admits_normalized` (four). The two code-only mutants fail the function's own postcondition: `normalized` and `from_bindings`. This matches TC-950.
- **The proofs are not vacuous.** With `assert(false)` added to all ten theorems at once, 430 verified, 10 errors, one at each site. Each ensures mutation fails: a strict lower bound or a shift of +1 in `member`; `||` to `&&` in the least-scale clause; `k + 1 == zeros`; the sign clause flipped in `lemma_normalized` and in `row_normalized`; scale 0 excluded from `from_bindings_post`; `lemma_zeros_bound` made strict below 2^127 (the call for `i128::MIN` then fails its precondition). Each code mutation fails too: the 16-zero stage dropped from code and spec; `shift >= 38` in code and spec (the 10^38 edge); `MAX_NATIVE_POWER = 39` (array bound); a negative scale clamped to 0; `checked_mul` replaced by `wrapping_mul` (the overflow of coefficient times power); `POWERS_OF_TEN[3]` used for the 2-zero stage; a negated quotient. Shifting the assumed table spec by one power breaks `admits_normalized` and `normalized_pair`, so the assumption carries weight.
- **Gate tests.** `verus_proofs_verify --ignored` passes at head and fails with `assert(false)` in `lemma_normalized`. A no-op mutant added to the ir1554 script is reported SURVIVED and the script exits 1, which `verus_mutants_are_killed` asserts against. A default `cargo test` lists both gate tests as ignored.
- **Host gates at head.** fmt-check, build-no-std, docs and deny pass. `cargo test --no-fail-fast`: 608 passed, 0 failed, 11 ignored. With `--features test-support`: 621 passed, 0 failed, 12 ignored. Main has one test fewer in each (the new AC-15 test). `cargo clippy --all-targets -D warnings` and `make lint` fail only on the known unused import `alloc::string::ToString` at src/json_object.rs:5, and clippy's errors match main 1f7c0d56 line for line.

## Verdict

Clean. The proof is sound for the stated domain and is not vacuous. It changes no compiled line, and its trusted base is exactly what Behavior 11 and docs/verus-proofs.md list. None of the earlier lane findings (#175, #179, #181) comes back in the code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/native_decimal.rs (diff) | examined | cfg_attr verus_spec on admits, admits_normalized, from_retained, from_bindings, coefficient, scale, normalized, normalized_pair, strip_zeros; external_body + ensures powers_table on POWERS_OF_TEN; verus_verify(external) on NativeDecimalType::new and to_decimal |
| src/native_decimal/verus_proofs.rs | examined | lemma_normalized, lemma_admits_normalized, row_from_bindings, row_normalized and their helper lemmas |
| scripts/ir1554_verus_mutants.py | examined | 12 mutants, 10 code-and-spec, 2 code-only |
| tests/verus_gate.rs | examined | FR-402-AC-13 and FR-402-AC-14 tags on the ignored gate tests; ir1554 script added to the loop |
| src/native_decimal/tests.rs | examined | the_power_table_holds_every_power_of_ten_inside_i128 |
| Makefile, docs/verus-proofs.md | examined | verus-mutants target; POWERS_OF_TEN row in the trust table |
