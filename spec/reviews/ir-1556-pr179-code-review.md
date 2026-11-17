---
id: SR-17940486709564847048047
title: "IR-1556 PR #179 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #179 at 56d9d8b0 (diff origin/main...HEAD, base 16eba89a, 10 files, +247/-10); IR-1556; src/native_integer.rs, src/native_integer/verus_proofs.rs, src/verus_kernel_model.rs, src/verus_std_specs.rs, scripts/ir1556_verus_mutants.py, scripts/ir1544_verus_mutants.py, Makefile, docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-957
    type: reviews
---

## Summary

Ticket: IR-1556. Code review with the rust-review lane folded in, scoped to `git diff origin/main...56d9d8b0`: 10 files, +247/-10, matching the brief. Merged #175 (IR-1544, 16eba89a) is the merge base and was not re-reviewed.

The one edit that touches behaviour. In `native_div_rem`, `quotient.and_then(|q| q.checked_sub(1))` became `match quotient { Some(q) => q.checked_sub(1), None => None }`, and `floor_q.and_then(|q| q.checked_add(1))` became the same `match` with `checked_add`. `Option::and_then` is defined as exactly this match. Each arm is the same: `None` maps to `None`, and `Some(q)` maps to the same checked call. Nothing else in the function moved. The closure was the last expression, so there is no short-circuit order to change, and the checked calls keep the same overflow behaviour. There is no error mapping in the function. Everything else in production files is ghost-only: one `cfg_attr(verus_keep_ghost, verus_spec(...))` attribute, one `#[cfg(verus_keep_ghost)] proof!` statement and one `#[cfg(verus_keep_ghost)] mod verus_proofs;`. `verus_keep_ghost` was already in Cargo.toml's `check-cfg`. `Cargo.toml` and `Cargo.lock` are unchanged. `KERNEL_VERSION` is unchanged. The only caller, `native_division`, returns `Undefined(DivisionByZero)` before it calls `native_div_rem`, so the `requires divisor != 0` holds at that call site. That guard is tested by the host differential, not proved by Verus, and FR-409 Behavior 24 says so.

Escape hatches. The new proof file has no `assume(`, `admit(`, `external_body` or axiom. The trusted base grows by two `assume_specification`s, both checked against core semantics:

- `i128::wrapping_rem` (src/verus_std_specs.rs). For `y != 0` it gives `rust_rem(x, y)`, and `0` for `i128::MIN % -1`. This matches core, where `wrapping_rem` panics only on a zero divisor. The `requires y != 0` is no weaker than core.
- `<DivisionProfile as PartialEq>::eq` (src/verus_kernel_model.rs) is equality of variants. `DivisionProfile` is a fieldless `#[derive(PartialEq)]` enum, so this is what the derived impl does. It comes with an `external_type_specification` of the enum.

Reviewer runs, in a private detached worktree under `flock /tmp/cargo-slot.lock`, with Verus already installed locally:

- `make verus` printed `verification results:: 221 verified, 0 errors` with 0.56 GiB peak RSS. The measured wall time (726 s) includes waiting for the lock and the dependency build, so it is not a solver time.
- `make verus-mutants`: the IR-1544 set was 18 of 18 killed, and the IR-1556 set (unmutated copy verified) was 10 of 10 killed. Every kill was `error: postcondition not satisfied`, and none was rlimit or UNRELIABLE.
- Reviewer probes, my own list. They ran on a scratch copy and were applied one at a time:
  - Vacuity. `assert(false)` fails with `assertion failed` at each place it was inserted: the end of `lemma_truncating`, the end of `lemma_mod_neg_bound`, the entry of `native_div_rem` after the lemma call, the floor-return branch, and the Euclidean tail. So the preconditions are not contradictory, and both adjusting branches can be reached.
  - Ensures mutants. Each fails `postcondition not satisfied`: floor and Euclidean windows swapped; truncating window keyed on the divisor's sign; Euclidean window `0 < r` (zero excluded); Euclidean top `< |b| - 1`; floor negative window `b < r < 0`; the absent quotient keyed to `MIN / 1`; the absent-quotient remainder `== 1`; the identity `a == q*b - r`.
  - Zero divisor. Dropping `requires divisor != 0` fails with `precondition not satisfied`.
  - Code mutant. A remainder off by one (`wrapping_rem(..).wrapping_add(1)`) fails a postcondition.
  - Two code edits survive, and both are equivalent mutants, as expected. `None => Some(0)` in the floor step cannot be reached, because the quotient is `None` only for `MIN / -1`, whose remainder 0 returns early. `floor_r > 0` in place of `>= 0` changes nothing, because `floor_r` is never 0 on that path.
- Gates at 56d9d8b0: see Verdict.

## Verdict

The proof is real and attached to the production function. It verifies, it is not vacuous, and it fails when the laws, the windows, the MIN / -1 case, the zero-divisor precondition or the code are mutated. The one rewrite is semantically identical to the code it replaces. The two new trusted assumptions are correct and justified. Every mutant kill is a failed postcondition, so #175 FND-001 (rlimit kills) is not repeated. No pin and no tool version appears in the diff, so #175 FND-002 is not repeated. No hard-coded dependency list is added, so #175 FND-003 is not repeated. One low finding: a malformed row in the trust-base table of docs/verus-proofs.md.

Gates at head, with CARGO_TARGET_DIR in the worktree:

- clippy `--locked --all-targets -D warnings`: passes. `make lint`: passes.
- build-no-std, docs and cargo deny: pass.
- fmt-check fails only on `tests/qsl2171_undefined_no_element.rs` and `tests/qsl2326_live_occurrences_limit.rs`, both unchanged by this PR.
- `cargo test --locked --no-fail-fast`, lib: fails `equality::tests::kernel_result_bearing_evaluation_returns_relation_plan_and_charge_order` and `plan_reservation_refusals_are_atomic_before_pair_charges`.
- The same run also fails `pairs_without_a_key_stay_unkeyed_and_refuse` (20 passed, 1 failed) and `tests/union.rs::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences`.
- With `--features test-support`, the lib additionally fails `equality::tests::pair_and_result_refusals_stop_at_the_first_denied_charge`.
- None of these files is in the diff. In a fresh worktree at origin/main 16eba89a, the same runs fail the identical set: the 2 fmt files, the 2 lib tests, `tests/composite_declarations.rs::pairs_without_a_key_stay_unkeyed_and_refuse`, `tests/union.rs::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences`, and the third lib test under `test-support`. So none of these failures is attributable to this PR. Main is red on 3 more items than the brief listed.
- `tests/ir1541_native_integer_divide.rs` passes 6 tests, with 1 ignored. The `native_integer` lib tests pass.
- git status is clean.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The trust-base table in docs/verus-proofs.md has two columns (`File`, `Assumption`). The edited `src/verus_kernel_model.rs` row ends `... that is FR-359 and its Kani gate. \| The derived equality of `DivisionProfile` compares variants.`, so the new assumption sits in a stray third cell after the closing pipe. Renderers drop that cell or show it as text outside the table, so the new trusted assumption is not visibly listed. Move the sentence inside the Assumption cell. | docs/verus-proofs.md:40 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/native_integer.rs native_div_rem and_then -> match (two sites) | examined | `match quotient { Some(q) => q.checked_sub(1), None => None }` / `match floor_q { Some(q) => q.checked_add(1), None => None }` |
| src/native_integer.rs verus_spec attribute and proof! call | examined | `#[cfg_attr(verus_keep_ghost, vstd::prelude::verus_spec(r => requires divisor != 0, ensures verus_proofs::div_rem_post(profile, dividend, divisor, r)))]` |
| src/native_integer.rs native_division zero guard | context_only | `if divisor.0 == 0 { return Err(Stop::Undefined(Undefined::DivisionByZero)); }` |
| src/native_integer/verus_proofs.rs in_window / div_rem_post | examined | Truncating window by the dividend's sign, floor window by the divisor's sign, Euclidean window `0 <= r < abs(b)`; `r.0 is None <==> (a == i128::MIN && b == -1)`; `Some(q) ==> a == q*b + r.1`; `None ==> r.1 == 0` |
| src/native_integer/verus_proofs.rs lemma_truncating, lemma_mod_neg_bound | examined | Facts of the truncating `/` and `%`: the identity, the remainder bound and sign, the quotient range, and both shifted identities |
| src/verus_std_specs.rs i128::wrapping_rem | examined | `r == i128_wrapping_rem(x, y)`, requires `y != 0`, 0 for MIN % -1 |
| src/verus_kernel_model.rs DivisionProfile eq | examined | `r == (*a == *b)` on a fieldless derived enum |
| scripts/ir1556_verus_mutants.py, scripts/ir1544_verus_mutants.py run(), Makefile verus-mutants | examined | 10 mutants; the kill counts only SEMANTIC lines; `run(MUTANTS)` refactor |
| docs/verus-proofs.md | examined | trust-base table rows |

## Dispositions

Round 1, fix head 2be4ec49 (reviewed head 56d9d8b0); base still origin/main 16eba89a. Reviewer re-runs in a fresh private worktree under flock /tmp/cargo-slot.lock. `cargo test --locked --test verus_gate -- --ignored`: 2 passed (verus_proofs_verify, verus_mutants_are_killed), 98.7 s wall. Without the ignore flag the two gate tests are skipped (0 passed, 2 ignored), so the default test path is unchanged. Non-tautology probes: the Euclidean window changed to `0 < r` made verus_proofs_verify FAIL (postcondition not satisfied, 220 verified, 1 errors); an extra equivalent mutant in scripts/ir1556_verus_mutants.py made verus_mutants_are_killed FAIL (reviewer-equivalent-survivor: SURVIVED). Both edits were reverted, and git status is clean. `quoin matrix`: FR-408-AC-13, FR-408-AC-14, FR-409-AC-11 and FR-409-AC-12 are tagged-by-ignored-test, and FR-408-AC-15 is tagged. Gates at 2be4ec49: clippy --locked --all-targets -D warnings, make lint, build-no-std, docs and deny pass. fmt-check differs only in tests/qsl2171_undefined_no_element.rs and tests/qsl2326_live_occurrences_limit.rs. The cargo test failure set is the one that fails identically on origin/main: 2 lib equality tests, plus 1 more with test-support; composite_declarations::pairs_without_a_key_stay_unkeyed_and_refuse; union::ieee_union_payload_has_no_key_and_sequence_keeps_occurrences. tests/ir258_native_rational.rs passes 16. The diff 56d9d8b0..2be4ec49 adds no pin and no tool version. The committed spec/reviews/ir-1556-pr179-*.md copies are byte-identical (cmp) to the reviewer's round-0 files.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: / `src/verus_kernel_model.rs` / `Meter::charge_units` appends an admitted request ... that is FR-359 and its Kani gate. The derived equality of `DivisionProfile` compares variants. / |
