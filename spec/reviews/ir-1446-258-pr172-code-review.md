---
id: SR-17940486709564847048017
title: "IR-1446 / IR-258 PR #172 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-15
scope: "agent-ix/filament-exact PR #172 at 5098e906, git diff origin/main...HEAD; IR-1446, IR-258; src/native_float/kani_proofs.rs, src/native_fraction.rs, src/native_rational/kani_proofs.rs, src/native_rational/reference.rs, tests/ir1446_native_float.rs, tests/ir258_native_rational.rs, scripts/ir1446_native_float_kani_mutants.py, scripts/ir258_u8_tier_mutants.py"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
---

## Summary

Tickets: IR-1446 (float), IR-258 (rational). This is a code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (12 files, +586/-159).

Coder claims checked against the code:

- Float harnesses (`src/native_float/kani_proofs.rs`). The arithmetic, fused, square-root and divide harness macros hold the outcome in `core::mem::ManuallyDrop`, match on `&*outcome`, and `check` takes `&Result<..>`. No file under `src/native_float.rs` changed, so the kernel is unchanged.
- `fused_multiply_add_expected` is not a tautology. It reads only the operand bits. For an infinite addend it returns `c`; inputs that are NaN are excluded first by `nan_fused_multiply_add`, which covers an infinite product with an opposite-signed infinite addend and 0 x inf. For a zero product with a finite nonzero `c` it returns `c`. For a zero product with a zero addend it returns `c` when the signs agree, else +0. Every other input goes to the host `mul_add`, which is CBMC's model. Nothing on the expected side calls the native body. These are the IEEE 754 results under round-to-nearest; `check` compares bits only for `NearestEven` and `Exact`.
- The two tool-fault harnesses run the model alone. Their claims are correct IEEE facts: `x*y + inf = inf` for finite x, y, and `0*b + c = c` for finite b and nonzero c.
- `gcd8` (`src/native_fraction.rs`) is a binary gcd on the odd parts with the common power of two restored. I re-derived the 16-step bound: each non-terminal step at least halves the product of the pair, and that product is below 2^16. The panic surface: `large - small` cannot underflow; `trailing_zeros` is applied only to nonzero values, so every shift is below 8; `divisor << shift` cannot overflow because the gcd times 2^shift is at most min(left, right); and `reduce` divides by a nonzero gcd. The function has no unsafe code, no loop and no `cfg(kani)`. `reduce` is one source with an ordinary data branch. Its only caller is `native_rational.rs`; Decimal and quantity do not call it, so their behaviour is unchanged.
- I checked equivalence outside Kani independently. A host simulation of the shipped step function over all 65,535 nonzero `u8` pairs against `math.gcd` gives 0 mismatches at 7 and at 16 steps, and 778 at 6. So 7 is the minimum and steps-6 is distinguishable. The host sweep `the_u8_tier_reduces_every_pair_as_euclid_does_across_its_threshold` passes. `scripts/ir258_u8_tier_mutants.py` (host) KILLED all six mutants (steps-4, steps-6, threshold-u16-narrowed, shift-dropped, even-parts-kept, difference-not-odd). The existing owner differential (AC-9) passes on the tiered kernel.
- `reference::reduce_narrow` asserts its range (`|n| <= 255`, `0 < d <= 255`) instead of assuming it. The CG-domain unreduced results are at most 162/81, so the assert never fires there. The full-i64 rows pass `wide = true`.

Gate, run in a private detached worktree under `flock /tmp/cargo-slot.lock`:

- `cargo fmt --check`: clean.
- `cargo clippy --locked --all-targets -- -D warnings`, with `--features test-support`, and `--lib --target thumbv7em-none-eabi`: all clean. Clippy does not see `cfg(kani)` code.
- `cargo test --locked`, default and `--features test-support`: all pass.
- Kani spot checks under `systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0`:
  - `float32_add_exact`: VERIFIED, 3 of 3 covers, 34.8 s (record: 35.3 s).
  - `native_fraction_gcd8_matches_euclid_over_u8`: VERIFIED, 2 of 2 covers, 9.0 s (record: 9.2 s).

## Verdict

No kernel defect was found. The drop-glue fix, the fused rule and the u8 tier are correct, and the tier preserves behaviour for every input. One medium finding: the `Exact` float harnesses' numeric-result claim has no vacuity evidence. Two low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `Exact` float rows (12 newly VERIFIED, for example `float32_add_exact`) claim 'result bits under a completed `Exact`' (TC-953 harness paragraph), but no evidence shows that claim can fail. `check` accepts `Ok(Outcome::Refused(_))` for every non-NaN input under `Exact` without requiring the result to be inexact. A completed `Exact` result is compared with the host NearestEven bits, so an 'Exact returns the rounded value' mutant also passes. The 'a numeric result is reached' cover is on `expected.is_some()`, which a kernel that always refuses also reaches. The only harness mutant recorded for these rows is `nan-payload-cleared`, which exercises the NaN path only. So a kernel that refuses every non-NaN `Exact` input verifies, and the `assert_eq!(bits, host)` under `Exact` has no shown vacuity check (FR-405 Behavior 20: 'a cover that the result path is reachable'). Fix: add a cover for `Ok(Completed(_))` with `expected.is_some()` under `Exact`, and a harness mutant on the numeric `Exact` path (for example 'Exact refuses every finite result'). Alternatively, record the `Exact` rows as covering only the NaN outputs and that only `Exact` refuses. | src/native_float/kani_proofs.rs:360, src/native_float/kani_proofs.rs:386-389, spec/test-cases/TC-953-native-float-carrier-matches-owner-and-verifies.md:135 |
| FND-002 | low | `fused_multiply_add_expected` returns +0 for a zero product plus a zero addend of opposite sign under every spelling. The docstring and TC-953 call this 'the signed zero the sum rule picks', but under `TowardNegative` the rule picks -0. This is harmless today because `check` compares bits only for `NearestEven` and `Exact`. If directed bits are ever compared, the expected side becomes wrong without warning. Scope the docstring to round-to-nearest, or take the rounding mode and return -0 for `TowardNegative`. | src/native_float/kani_proofs.rs:166-189 |
| FND-003 | low | `scripts/ir1446_native_float_kani_mutants.py` dropped its `timeout=700`, and the per-mutant limit now comes only from `KANI_RUNNER`. The ignored test `every_harness_mutant_fails_its_harness_under_kani` runs the script without setting `KANI_RUNNER`. A mutant that makes the solver hang therefore runs without a bound and holds the cargo lock. Have the test, or the script by default, set a bound (for example `timeout 1800`), or fail fast when `KANI_RUNNER` is unset. | scripts/ir1446_native_float_kani_mutants.py:63-67, tests/ir1446_native_float.rs:1057-1065 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| float harness macros (ManuallyDrop, check by reference) | examined | src/native_float/kani_proofs.rs:337-560 | let outcome = core::mem::ManuallyDrop::new(evaluate_native_float(...)); $module::check(rounding, &*outcome, &meter, ...) |
| fused_multiply_add_expected | examined | src/native_float/kani_proofs.rs:166-189 | an infinite addend gives the addend, and a zero product gives the addend (or the signed zero ...) |
| tool-fault harnesses | examined | src/native_float/kani_proofs.rs:836-858 | tool_fault_fma_model_infinite_addend_f32, tool_fault_fma_model_zero_product_f32 |
| gcd8 / step8 / reduce u8 tier | examined | src/native_fraction.rs:59-170 | pub(crate) fn gcd8(left: u8, right: u8) -> u8 |
| reference reduce_narrow / gcd_narrow | examined | src/native_rational/reference.rs:56-87 | both magnitudes are at most 255 (asserted, not assumed) |
| tier Kani harnesses and commutative_from_i64_parts | examined | src/native_rational/kani_proofs.rs:515-560 | native_fraction_gcd8_matches_euclid_over_u8, native_fraction_reduce_u8_tier_matches_euclid |
| host sweep and evidence tests | examined | tests/ir258_native_rational.rs:1015-1045, tests/ir1446_native_float.rs:815-1082 | the_u8_tier_reduces_every_pair_as_euclid_does_across_its_threshold |
| mutant scripts | examined | scripts/ir258_u8_tier_mutants.py, scripts/ir1446_native_float_kani_mutants.py | MUTANTS, ROWS, KANI_RUNNER |

## Dispositions

Round 1, verified against fix commit cd71d77b (`git diff 5098e906..cd71d77b`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `check` gains the cover 'a numeric result completes' (`Ok(Completed)` with `expected.is_some()`); under `Exact`, a completed numeric result must have empty flags; a refusal must be `IeeeNotExact` with a non-empty would-be set that does not contain `invalid`. So a kernel that refuses exact results fails on the empty would-be set, and returning the rounded value fails the no-flag check. New host mutant `exact-refuses-everything` (50 host mutants). It is listed against all 8 `Exact` rows. `exact-returns-rounded` is listed against 6 of them. The other 2 (`float32_square_root_exact`, `float64_multiply_exact`) returned NO-VERDICT at the 1800 s cap, and TC-953 and Behavior 20 (SHOULD) record that honestly. Those 2 rows keep non-NaN mutant credit through `exact-refuses-everything`, which drives the numeric refusal path. The evidence test enforces the per-row mutant lists. |
| FND-002 | fixed | The docstring now says +0 is the sum rule's zero under round-to-nearest only, that TowardNegative gives -0, and that `check` compares bits for NearestEven and Exact alone. |
| FND-003 | fixed | The script defaults `KANI_RUNNER` to `timeout 1800`, and the ignored test passes the caller's runner or `timeout 1800`. |
