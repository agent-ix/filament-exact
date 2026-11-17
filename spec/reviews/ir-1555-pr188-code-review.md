---
id: SR-17940486709564847048056
title: "IR-1555 PR #188 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #188 at 4e54d122 (diff origin/main...4e54d122, 12 files, +2917/-24); IR-1555; src/native_decimal/operations.rs, src/native_decimal/operations/verus_proofs.rs, src/native_decimal/verus_proofs.rs, src/verus_std_specs.rs, src/native_float.rs, src/native_integer.rs, scripts/ir1555_verus_mutants.py, tests/verus_gate.rs, Makefile, docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-403
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-951
    type: reviews
---

## Summary

Ticket: IR-1555. Code review with the rust-review lane. The reviewer re-ran everything in a private worktree under `flock /tmp/cargo-slot.lock`, with logs in the worktree's `target/`.

- **Compiled code is unchanged except for the two disclosed rewrites.** Every other added line in `src/native_decimal/operations.rs` is a `cfg_attr(verus_keep_ghost, ...)` attribute or one of its continuation lines, a `#[cfg(verus_keep_ghost)]` item or `proof!` statement, or the gated `verus!` block of closed spec accessors. `native_float.rs` and `native_integer.rs` only change a gated `mod verus_proofs` to `pub(crate)`. This was checked with a diff of non-ghost lines. Cargo.toml, Cargo.lock and KERNEL_VERSION are not touched.
- **Disclosure (b): the closure rewrites change no behaviour.** In `reduce_quotient`, `(magnitude <= 1 << 127).then(|| (magnitude as i128).wrapping_neg())` becomes `if magnitude <= 1 << 127 { Some(...) } else { None }`. The closure was pure and lazy, so the guard, the evaluation, the overflow behaviour and the `None` path are identical. In `expansion`, the closure `sized((c, k))` becomes the private fn `expanded(c, k)`. It has the same body, `first` is still preferred over `second`, and the same `(0, None)` fallback applies. Host tests pin both. Five host mutants each fail existing tests: `<` for `<=` on the least quotient, the negation dropped, the shift plus one, bits and digits swapped, and the second operand taken first. 2 to 5 failures each, in `operations_tests` (grid and random reference binding) and `tests/ir1508_native_decimal_ops.rs` (`native_and_owner_agree_over_the_boundary_set`, `charge_amounts_are_the_owners`, the limit sweeps).
- **Disclosure (c): the trusted base is correct.** Each new `assume_specification` matches `core`. `checked_neg` is `None` exactly at `i128::MIN`. `div_euclid` and `rem_euclid` require `y > 0`, and under that requirement Verus's int `/` and `%` are the floor and the nonnegative remainder, which is the Euclidean result. `MIN.div_euclid(1) = MIN` fits, and the only overflowing case (`MIN / -1`) is excluded by the requirement. `signum` gives 0 at 0 and -1 at MIN. `Ordering::reverse` is correct. The `u128_trailing_zeros` recursion gives 128 at 0, 0 for odd values and `1 + tz(i/2)` otherwise. `then_some` is eager, like `core`. `u64::from(bool)` gives 0 or 1. All are named in Behavior 41 and in the docs/verus-proofs.md trust table. There is no `assume(`, `admit(`, `external_body`, `axiom` or `opaque`. The new `external_type_specification` of `NativeDecimalCapacityRefusal` is transparent and matches the one-field struct, as in the earlier lanes.
- **Preconditions.** The entry points `admit` and `order_values` have no precondition. `settle` requires `well_formed()`. `admit` ensures it, and `AdmittedNativeDecimalOperation` is built at exactly one site (`admit`, operations.rs:836). The internal preconditions are all discharged by Verus at their call sites: `shifted` (shift <= 38), `reduce_quotient` (d != 0), the strips, `loss_record`, `round_units`, `rounding_tail`, `place_scaled` (working <= 2^33), `place_quotient` (canonical), `exact_tail` (scale <= T) and the comparators (nonzero). No precondition shrinks the domain.
- **Reproduced.** `make verus`: 630 verified, 0 errors, 13.8 s wall, 0.82 GiB peak. `make verus-mutants`: all five scripts, 22:34 wall, 0.84 GiB peak. The ir1555 script killed 53 of 53 (48 by a postcondition, 3 by an assertion, 1 by a precondition, 1 by an overflow), with no SURVIVED, UNRELIABLE or INVALID. The other sets are unchanged and all killed (126 in total).
- **The proofs are not vacuous.** `assert(false)` placed after the head ghost call fails in each of these: `admit`, `settle`, `order_values`, `compare_magnitudes`, `compare_shifted_magnitude`, `widening_mul`, `place_quotient`, `place_scaled`, `round_units`, `loss_record` and `reduce_quotient`, and inside both arms of `lemma_admitted` and in `lemma_rounds_unique`. That is 14 of 14. The extra mutants all fail a proof obligation, 18 of 18:
  - wrapping instead of checked at the i128 edges (the negation of MIN, the sum, the product, the shifted operand, the divide numerator);
  - 1.0 against 1.00 reported unequal (`(high, low) == (0, y)` returns Greater);
  - equal magnitudes at the same scale reported Less;
  - two zeros reported Less;
  - toward-negative ceils;
  - above half floors;
  - the divide shift off by one;
  - `exact_tail` and `shifted` edited in both code and spec (killed by `lemma_scaled_means` and the plan postconditions);
  - five edits to the specification alone: `cmp_int` reversed, toward-zero swapped, nearest-away toward zero, the quotient sign dropped, the combine scale the smaller one.
- **Gate tests.** A default `cargo test` lists `verus_proofs_verify` and `verus_mutants_are_killed` as ignored. `verus_proofs_verify --ignored` passes at head and fails with `assert(false)` in `order_values`. A no-op mutant added to the ir1555 script is reported SURVIVED and the script exits 1, which `verus_mutants_are_killed` asserts against.
- **Disclosure (d).** Most functions carry a mathematical postcondition, and their code-only mutants fail that statement. The functions whose spec copies the body (`capacity`, `lift_fits`, `tie`, the strips, `loss_record`) are mutated in code and spec, with one exception (FND-001). Every function Behavior 41 lists has at least one mutant.
- **Host gates at head.** All pass: fmt-check, `cargo clippy --all-targets -D warnings`, the same with `--features test-support`, `make lint` (including the no_std clippy), build-no-std, docs and deny. `cargo test --no-fail-fast` gives 613 passed, 0 failed, 9 ignored. With `--features test-support` it gives 626 passed, 0 failed, 10 ignored.

## Verdict

The proof is sound for the stated domain. It is not vacuous, its trusted base is exactly what Behavior 41 and docs/verus-proofs.md list, and the compiled code changes only by the two disclosed rewrites, which are behaviour-preserving and pinned by host tests. One medium finding: a mutant repeats the #179 code-only-on-a-copied-spec pattern, and its code-and-spec form only reaches the resource limit, which repeats the #175 rlimit-only pattern.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Mutant `five-ladder-entry-wrong` edits only the `FIVE_LADDER` table, but the `strip_fives` spec is a copy of its body with the ladder literals inlined. It is killed only by `strip_fives`'s own copy postcondition (operations.rs:556), not by a theorem. With the spec literal also edited, the run ends at the resource limit in `lemma_strip_fives` (verus_proofs.rs:869), which is no verdict. TC-951 says such functions are edited "both, so a theorem is what fails", and its row lists this mutant without "(both)". | scripts/ir1555_verus_mutants.py:101, src/native_decimal/operations.rs:556, src/native_decimal/operations/verus_proofs.rs:795-802, spec/test-cases/TC-951-native-decimal-operations-match-owner-path.md:125,140 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/native_decimal/operations.rs (diff) | examined | cfg_attr verus_spec on 27 functions; proof! lemma calls; gated verus! block (admission, order_by_value, well_formed, settlement); reduce_quotient conditional; expansion -> expanded |
| src/native_decimal/operations/verus_proofs.rs | examined | planners, rounding, loss record, placement, admission/decision, ordering specs and lemmas |
| src/native_decimal/verus_proofs.rs (diff) | examined | bits_of, digit_len, digits_of, lemma_digits_of |
| src/verus_std_specs.rs (diff) | examined | assume_specification of u64::from(bool), then_some, checked_neg, div_euclid, rem_euclid, signum, Ordering::reverse, u128::trailing_zeros |
| scripts/ir1555_verus_mutants.py | examined | 53 mutants |
| tests/verus_gate.rs | examined | FR-403-AC-15 / AC-16 tags on the ignored gate tests; ir1555 script in the loop |
| Makefile, docs/verus-proofs.md, src/native_float.rs, src/native_integer.rs | examined | verus-mutants target; trust table; pub(crate) gated proof modules |
