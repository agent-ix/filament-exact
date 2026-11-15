---
id: SR-17940486709564847047996
title: "IR-1541 PR #171 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-15
scope: "agent-ix/filament-exact PR #171 at 4002545d, git diff origin/main...HEAD; IR-1541; src/native_integer.rs, src/native_integer/division_proofs.rs, src/native_integer/kani_proofs.rs, src/native_integer/reference.rs, tests/ir1541_native_integer_divide.rs, tests/ir705_native_integer.rs, scripts/ir1541_native_integer_mutants.py, scripts/ir1541_run_kani.py, scripts/ir1541_ladder.py"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-957
    type: reviews
---

## Summary

Ticket: IR-1541. The PR adds `Divide { profile, member, dividend, divisor }` and `Modulo { dividend, divisor }` to `NativeIntegerOperation` and `NativeIntegerOperationKind`, derives their capacity in `NativeIntegerPlan::new`, admits binary operands through the existing interval check, and evaluates an admitted division through `plan_division`, which calls the existing plan-free `native_division` with the plan's result domain and then checks the value against the plan's capacity (`CheckedInvariant(NativeIntegerCapacityBreach)` otherwise). It adds 80 Kani harnesses (`division_proofs.rs`), a host differential (`tests/ir1541_native_integer_divide.rs`), a mutant script, a Kani runner and a ladder driver.

Reviewer reproduction at 4002545d (`CARGO_TARGET_DIR=/home/peter/dev/.review-targets/ir1541`, under `flock /tmp/cargo-slot.lock`):

- `cargo test --lib`: 258 passed, 0 failed, 1 ignored. The known `once_box_race` flake did not show.
- `cargo test --test ir1541_native_integer_divide --test ir705_native_integer`: 5 passed (1 ignored, the replay driver) and 10 passed.
- `cargo clippy --all-targets`: clean. `cargo fmt --check`: clean.
- Healthy replay of all 53 recorded mutant playbacks through `replay_a_decoded_playback` on the healthy crate (FND-001).
- Kani spot check (Kani as installed, cadical, 16 GiB scope): see Verdict.

Plan code, checked by reading:

- Capacity. Quotient: refused exactly when `dividend.lower == i128::MIN` and `-1` lies in the divisor interval; otherwise `[clamp_negative(M), clamp_positive(M)]` with `M` from `unsigned_abs`, so `|i128::MIN|` is `2^127` and clamps to `[i128::MIN, i128::MAX]`. Remainder `[-(B'-1), B'-1]` and `Modulo` `[0, B'-1]` with `B' = max(B, 1)`, saturating at `i128::MAX`. These match FR-409 Behavior 4 and 5. No division of a symbolic operand and no `Integer` is constructed in construction or admission (Behavior 6).
- Admission. `Divide` and `Modulo` join the binary arm, so the dividend and divisor are checked against their intervals with `OperandOutsideInterval { position }`, and unary operands fall through to `OperandShapeMismatch { operation: Divide | Modulo }`. A zero divisor in the interval is admitted.
- Kernel-checked per AD-006. The plan path is the only new native entry for division; the plan-free evaluators stay public and unchanged, as Behavior 12 requires. The plan route checks the interval and capacity before evaluation and returns typed refusals.
- Add, Subtract, Multiply and Negate are unchanged: `native_arithmetic` returns early only for `Divide`/`Modulo`; `calculate` keeps its four arms; the new `charge_arithmetic_prefix` arm is unreachable; `charge_schedule` routes only the two new kinds to `charge_division_schedule`. The FR-384 tests (`tests/ir705_native_integer.rs`, only `unreachable!` arms added) pass.
- Harnesses. The expected side is the window relation `a - q * b` in `W` with products ordered quotient then divisor, in `i128` up to width 64 and through `reference.rs` at width 128. It contains no `/`, `%` or `*_div`/`*_rem`/`div_euclid`/`rem_euclid`. The remainder and `Modulo` companion quotient comes from the plan-free evaluator and is itself pinned by the relation. Each harness docstring names its outputs and its assigned mutant.
- Rename. The tests import `filament_exact`; env vars are `IR1541_*`; no `quire_exact` or `QUIRE_` remains in the diff.

## Verdict

Not mergeable as is, because of FND-001. The plan, admission and evaluation code look correct. They match FR-409 Behavior 1 to 12, the differential and the FR-384 tests pass, and the 15 host mutants are real text edits of the scratch copy (`prepare` asserts that the old text occurs once and that the new text landed) and each fails a test related to its site. The defect is in the AC-9 replay tooling: its "healthy" replay did not run the healthy crate, so the REPLAY-SAME / REPLAY-DIFFERS column in TC-957 is wrong. The fix is cheap: isolate the target directory per copy and re-run only the host replays from the existing logs. No Kani re-run is needed. FND-002 is low.

Kani spot check (2 harnesses, reviewer run, cadical, 16 GiB scope): `modulo_plan_derives_capacity_and_never_refuses` VERIFIED in 4.6 s with 3 of 3 covers (recorded: 3.0 s, 3/3). `trunc_quotient_sym_w8` VERIFIED in 51.4 s (recorded: 48.6 s). Its 3 satisfied covers are zero divisor, completed quotient and quotient out-of-domain. The 3 unreachable covers are the remainder and `Modulo` covers of the shared `evaluation` function, which supports FND-002.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The AC-9 host replay never ran the healthy crate. `replay()` runs `cargo test` in the repo (`ROOT`) and then in the mutant copy, with the inherited environment. With a shared `CARGO_TARGET_DIR`, cargo hashes the two path packages the same and records dep-info relative to the package root, so the repo build counts as fresh and reuses the previous mutant's test binary. Evidence: row 1 (`swap-truncating-floor`, `trunc_quotient_sym_w8`, input `-20 div 8`, domain `[-2,-1]`) records healthy `plan=Refused(DivisionOutOfDomain)` beside `exact=Completed(-2)`. The real healthy crate gives `plan=Completed(-2)`. Row 51 (`modulo-truncating`, `modulo_const_7_w64`) records healthy `plan=Completed(6)` outside the domain `[-2,-1]`, which is the previous row's `result-interval-skipped` mutant. Re-run on the healthy crate, every one of the 50 evaluation playbacks gives healthy plan == exact. In the recorded mutant column, all 23 REPLAY-SAME rows give a mutant outcome that differs from `divide`/`modulo` over `Integer` (21 differ, 2 panic: zero guard removed). So the 23 are misclassified by the tooling. They are not decoder limits, and not surviving or equivalent mutants. Give each copy its own target dir (or `--target-dir <copy>/target`), re-run the 53 replays from the existing `ir1541-logs/mutant-*.log` playbacks, and regenerate the labels. | scripts/ir1541_native_integer_mutants.py:217-226, scripts/ir1541_native_integer_mutants.py:244-252 |
| FND-002 | low | The runner's `Covers` cell prints Kani's raw `N of 6` (for example `3/6`, `2/6`). The shared `evaluation` function compiles the covers of every member combination into each harness, so 3 or 4 of the 6 are structurally unreachable. The runner checks only the shape's required covers (`required_covers`), but the record then shows `3/6` beside a Status that says 'every cover reached'. Print the required covers as reached/required (for example `3/3 required`) or name the unreachable ones as belonging to other combinations. | scripts/ir1541_run_kani.py:84-90, src/native_integer/division_proofs.rs:196-270 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| FR-409-AC-1 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:135 | `NativeIntegerOperation` has `Divide { profile, member, dividend, divisor }` and `Modulo { dividend, divisor }` ... unary operands return `OperandShapeMismatch` naming `Divide` or `Modulo`, each before any charge. |
| FR-409-AC-2 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:136 | A quotient plan whose dividend interval contains `i128::MIN` and whose divisor interval contains `-1` returns `CapacityExceeded { operation: Divide }` under each profile ... Each capacity equals Behavior 5's interval ... |
| FR-409-AC-3 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:137 | On the Behavior 21 grid ... evaluating the admitted operation returns the same outcome kind, value and refusal payload as `divide` or `modulo` over `Integer` and as the plan-free evaluator ... |
| FR-409-AC-4 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:138 | `charge_native_integer_schedule` over an admitted `Divide` or `Modulo` operation produces the events and counters of `NativeIntegerSchedule::Division` or `Modulus` ... The existing four arithmetic plans' ... are unchanged ... |
| FR-409-AC-5 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:139 | The plan harness of Behavior 13 is present ... each harness names its assigned harness mutant from Behavior 22's table. |
| FR-409-AC-6 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:140 | Code review of each harness finds an expected side that shares no function with the production division body ... Code review of plan construction and admission finds no division of a symbolic operand and no `Integer` construction. |
| FR-409-AC-9 | examined | spec/functional/FR-409-kernel-checked-native-integer-divide-and-modulo-plan.md:143 | Each mutant of Behavior 22 fails the host differential at its distinguishing input ... An evaluation-harness playback replayed through `divide` or `modulo` over `Integer` gives a different value or outcome from the mutant's ... |
| scripts/ir1541_native_integer_mutants.py | examined | scripts/ir1541_native_integer_mutants.py:1-275 | Buildable mutants of FR-409 Behavior 22 / TC-957, host and Kani. |
| scripts/ir1541_run_kani.py | examined | scripts/ir1541_run_kani.py:1-106 | Run FR-409 Kani harnesses one job at a time and append a results TSV. |
