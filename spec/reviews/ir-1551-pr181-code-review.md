---
id: SR-17940486709564847048050
title: "IR-1551 PR #181 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #181 at 182ab1ce (diff origin/main...HEAD, base 62330f44, 13 files, +2856/-8); IR-1551; src/native_float.rs, src/native_float/verus_proofs.rs, src/ieee.rs, src/lib.rs, src/native_rational.rs, src/verus_kernel_model.rs, scripts/ir1551_verus_mutants.py, scripts/ir1544_verus_mutants.py, tests/verus_gate.rs, Makefile, docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-953
    type: reviews
---

## Summary

Ticket: IR-1551. Code review with the rust-review lane folded in, scoped to `git diff origin/main...182ab1ce`. The diff has 13 files, +2856/-8, in one commit on main 62330f44.

Compiled code is unchanged. Every added line in `src/native_float.rs`, `src/ieee.rs` and `src/lib.rs` is one of three kinds: a `#[cfg_attr(verus_keep_ghost, ...)]` attribute; a `#[cfg(verus_keep_ghost)] proof! { ... }` statement; or an item under `#[cfg(verus_keep_ghost)]`. The cfg items are the `vstd` imports, the `verus!` block in `ieee.rs` and `mod verus_proofs`. No line was removed from these three files. In `src/native_rational.rs` the visibility of `#[cfg(verus_keep_ghost)] mod verus_proofs` changed to `pub(crate)`, which is also ghost-only. `feature(proc_macro_hygiene)` is enabled only under `cfg_attr(verus_keep_ghost)`. The non-Verus build passes clippy, lint, no_std, docs and deny, so the loop-statement attribute does not leak. `Cargo.toml`, `Cargo.lock`, `KERNEL_VERSION` and `rust-toolchain.toml` are unchanged.

Escape hatches. The new proof file has no `assume(`, `admit(`, `external_body` or axiom of its own. It calls `vstd::std_specs::bits::axiom_u64_leading_zeros`, a lemma from the prover library. The trusted base grows by these items:

- `assume_specification <RoundingMode as PartialEq>::eq` is equality of variants. `RoundingMode` is a fieldless enum with derived `PartialEq` (src/decimal.rs:168), so this is what the derived impl does.
- `external_type_specification` of `RoundingMode`, `IeeeWidth` and `IeeeFlag`. These are transparent (no `external_body`), so they add no behaviour.
- `verus_verify(external)` on `IeeeFlag::as_str` and `IeeeFlags::iter`. No proof uses either.
- Pre-existing items the proofs use: the `u128::leading_zeros` spec in src/verus_std_specs.rs (128 for zero, otherwise one less than for `i/2`, which is correct), and the library's specs of `u64::leading_zeros`, `u128::is_multiple_of`, `u128::from` and the shifts.

The float semantics were checked against IEEE 754 binary32 and binary64:

- `decode`: the bias is `2^(e-1) - 1`; a subnormal has exponent `quantum_min = 1 - bias - f`; a normal has the implicit bit and exponent `biased - bias - f`. A NaN is signaling when the quiet bit is clear.
- `rounds_to` gives the nearest value with ties to even, the nearest with ties away, toward zero, and toward positive or negative by sign. `Exact` maps to nearest-even.
- `lemma_grid_round` proves that rounding is unique on the two grids.
- An overflow goes to infinity under the nearest spellings. It goes to the largest finite value toward zero, and by sign under the directed spellings. It raises overflow and inexact.
- Tininess is detected after rounding: the value is rounded at half-quantum with an unbounded exponent and compared with `2^emin`. Underflow is raised when the result is tiny and inexact, the IEEE default.
- Divide and square root keep at least `f + 3` bits, so `round` sees two guard bits plus a sticky bit.
- The fused multiply-add sum is exact (`lemma_sum`) and is rounded once.

Reviewer runs, in a private detached worktree under `flock /tmp/cargo-slot.lock`:

- `make verus`: `393 verified, 0 errors`, 8.35 s wall, 0.66 GiB peak RSS.
- `make verus-mutants`, all three scripts: 4:05.65 wall, 0.66 GiB peak. The IR-1544 set killed 18 of 18, IR-1556 killed 10 of 10 and IR-1551 killed 17 of 17. Every kill was a postcondition, an assertion or a loop invariant (`invariant not satisfied` at loop entry or end). None was rlimit or UNRELIABLE.
- `cargo test --locked --test verus_gate -- --ignored`: 2 passed. Without `--ignored`, 2 are ignored, so default `make test` does not run Verus.
- Gate probes. With `assert(false)` added to `lemma_round_at`, `verus_proofs_verify` FAILED. With a no-op mutant added to the IR-1551 script, `verus_mutants_are_killed` FAILED and the mutant was reported SURVIVED. Both edits were reverted, and `git status` is clean.
- Vacuity. `assert(false)` at the end of 15 theorems failed in all 15 (378 verified, 15 errors): `row_divide`, `row_square_root`, `row_multiply`, `row_fused_multiply_add`, `lemma_round`, `lemma_sum`, `lemma_plan_divide`, `lemma_plan_square_root`, `lemma_round_at`, `lemma_grid_round`, `lemma_decode_rounded`, `lemma_sqrt_step`, `lemma_sqrt_end`, `lemma_justify` and `lemma_normalised`. `assert(false)` inside the `integer_square_root` loop body failed, and so did one after the loop.
- Loop invariant strength. Dropping any one conjunct of `sqrt_inv` breaks the proof. Without the upper bound, `lemma_sqrt_end` has a precondition failure. Without the alignment, `lemma_sqrt_step` has an assertion failure. Without the lower bound, a postcondition fails.
- Theorem mutants that edit both the code and its specification, so that only a theorem can fail. Killed: nearest-away ties down; toward-positive ignores the sign; the sticky bit ignored at the half (`lemma_round_at`/`lemma_round_at_is_grid_round` fail); an overflow toward zero going to infinity; the carry not renormalised; underflow tested before rounding (`lemma_round` fails); the decode bias off by one; the subnormal exponent off by one (`lemma_decode_rounded` fails); the jam dropped in `sum` (`lemma_sum` fails); the divide sticky dropped (`lemma_plan_divide` fails); the product exponents subtracted (`lemma_plan_multiply` fails); `Exact` mapped to toward-zero; the square-root sticky dropped. Verified unchanged, which shows these are not covered by any theorem: divide by zero raising `invalid`, a signaling NaN read as quiet, and an infinite FMA product taking the addend's sign (FND-002).
- Gates at 182ab1ce: `cargo clippy --locked --all-targets -- -D warnings`, `make lint`, `make build-no-std`, `make docs` and `make deny` pass. `make fmt-check` differs only in tests/qsl2171_undefined_no_element.rs and tests/qsl2326_live_occurrences_limit.rs, which fail the same way on main.
- `cargo test --no-fail-fast`, default and `--features test-support`, at head and at main 62330f44. The head has one failure that main does not have (FND-001). The other failures are on main too: the 2 lib equality tests (3 with test-support), `composite_declarations::pairs_without_a_key_stay_unkeyed_and_refuse` and `union::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences`.

## Verdict

The proofs are sound and not vacuous, the trusted base is small and correct, and compiled code is unchanged. Not mergeable: FND-001 is a new test failure on the PR's own criterion, hidden because `make test` stops at the first failing target. FND-002 is evidence inflation in the mutant set. FND-003 is a docstring nit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | New test failure: `the_status_records_a_verdict_or_table_row_for_every_operation_and_width` (FR-405-AC-15) fails at head with `assertion failed: status.contains("No proof is claimed for a table-row combination")`. The PR rewrote that FR-405 Status sentence as "No Kani proof is claimed for a table-row combination", and the test passes on main 62330f44. The author's claim that make test fails only the two equality unit tests does not hold. `make test` runs `cargo test` without `--no-fail-fast`, so it stops after the lib target fails and never runs tests/ir1446_native_float.rs. Update the assertion, or keep the phrase, and run `cargo test --no-fail-fast`. | tests/ir1446_native_float.rs:973; spec/functional/FR-405-kani-tractable-native-float-carrier.md:114 |
| FND-002 | medium | 11 of the 17 IR-1551 mutants edit only the production code. The `ensures r == verus_proofs::<copy>` equality kills them, not a theorem. Three of them target behaviour that no theorem states and that Behavior 23 excludes (classify, NaN, decided branches): divide-by-zero-raises-invalid, signaling-nan-read-as-quiet and fma-infinite-product-takes-the-addend-sign. Their code-and-spec variants verify (393 verified, 0 errors). TC-953 still lists all three as kills behind Behavior 23. Drop the three, or mark copy-equality kills as such in TC-953 and the script. Add code-and-spec variants of the round_at and round mutants; the reviewer probes show `lemma_round_at` and `lemma_round` kill those. | scripts/ir1551_verus_mutants.py:114-139; spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:229 |
| FND-003 | low | The docstring of scripts/ir1551_verus_mutants.py says the mutants are of "FR-405 Behavior 22". The proof is Behavior 23; Behavior 22 is the Kani no-claim rule. | scripts/ir1551_verus_mutants.py:2-3 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/native_float.rs | examined | attributes and `#[cfg(verus_keep_ghost)]` ghost statements on decode, Fmt helpers, planners, integer_square_root, sum, round_at, round |
| src/native_float/verus_proofs.rs | examined | specifications, theorems and the rows row_divide, row_square_root, row_multiply, row_fused_multiply_add |
| src/ieee.rs, src/lib.rs, src/native_rational.rs, src/verus_kernel_model.rs | examined | flag-set specifications; proc_macro_hygiene under Verus; pub(crate) proofs module; RoundingMode eq assumption |
| scripts/ir1551_verus_mutants.py, scripts/ir1544_verus_mutants.py, tests/verus_gate.rs, Makefile | examined | 17 mutants; SEMANTIC adds invariant and shift errors; gate traces FR-405-AC-17/18 |
| tests/ir1446_native_float.rs:973 | examined | assert!(status.contains("No proof is claimed for a table-row combination")); |

## Dispositions

Round 1, fix head e2aa389b (reviewed head 182ab1ce; c20de78c fixed the AC-15 test). Base origin/main moved to a91a3710 (spec-only #183, FR-413/TC-961); the PR merges cleanly onto it. Reviewer re-ran everything in a fresh private worktree under flock /tmp/cargo-slot.lock, with logs in the worktree target/.
- `make verus`: 396 verified, 0 errors, 7.88 s wall, 0.67 GiB peak.
- `make verus-mutants`: 5:09 wall, 0.68 GiB peak. The three sets killed 18/18, 10/10 and 33/33. Every kill was a postcondition, precondition, assertion, overflow or loop invariant; none was rlimit, UNRELIABLE, SURVIVED or INVALID.
- `verus_gate --ignored`: 2 passed. Default `make test`: 2 ignored.
- `assert(false)` failed in all 16 theorems, `lemma_format_values` included (380 verified, 16 errors). Dropping the upper-bound or the alignment conjunct of `sqrt_inv` fails the proof.
- Gates: clippy --locked --all-targets -D warnings, make lint, build-no-std, docs and deny pass. fmt-check differs only in the two qsl tests.
- `cargo test --no-fail-fast`, default and test-support: the failures are the 2/3 lib equality tests, composite_declarations::pairs_without_a_key_stay_unkeyed_and_refuse and union::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences, all known on main. tests/ir1446_native_float.rs passes.
- The fix diff adds no pin and no tool version. src/native_float.rs, Cargo.toml, Cargo.lock and KERNEL_VERSION are unchanged.
- spec/reviews/ir-1551-pr181-*.md were byte-identical (cmp) to the round-0 files.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: assert!(status.contains("No Kani proof is claimed for a table-row combination")); (c20de78c). the_status_records_a_verdict_or_table_row_for_every_operation_and_width passes in both feature sets. |
| FND-002 | fixed | after: The three out-of-scope mutants are dropped. 31 of 33 mutants edit both the code and the spec, and two edit the sqrt loop, which has no spec copy. A theorem, an assertion, an overflow or a loop invariant kills each one. TC-953 states this split. |
| FND-003 | fixed | after: """Mutants of the native float arithmetic the Verus proofs cover (FR-405 Behavior 23, TC-953). |
