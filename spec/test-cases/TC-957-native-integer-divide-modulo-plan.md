---
id: TC-957
title: "The native integer Divide and Modulo plan matches divide and modulo and settles each Kani harness"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-409
    type: verifies
---
# TC-957: The native integer Divide and Modulo plan matches divide and modulo and settles each Kani harness

## Description

Verify FR-409-AC-1 through FR-409-AC-12. The procedure runs the plan path beside `divide` and `modulo` over `Integer` and beside the plan-free evaluators, inspects the harness and plan source, and records the Kani probe, the verdicts and the peaks. Each mutant below is a buildable variant of the named site in a scratch copy. Each names the input that fails it and the step at which it fails, and a mutant that no input distinguishes is not admitted. A timeout, memory exhaustion, missing cover or missing run is inconclusive and never a pass.

## Test Procedure

The grid is fixed before implementation from independent literals, not from the production body. Dividends and divisors are `0`, `1`, `-1`, `2`, `-2`, `7`, `-7`, `i64::MIN`, `i64::MIN + 1`, `i64::MAX`, `i128::MIN`, `i128::MIN + 1` and `i128::MAX`, crossed. Each pair is run under the truncating, floor and Euclidean laws for the quotient and the remainder, and for `Modulo`, with no result interval, a result interval containing the member and one excluding it.

1. For AC-1, construct a `Divide` plan for each profile and member and a `Modulo` plan over in-capacity intervals that contain zero in the divisor. Admit a member pair, a zero divisor, a dividend outside its interval, a divisor outside its interval and unary operands. Assert the typed error, its position, the operation it names, and that a recording `Meter` sees no charge.
2. For AC-2, construct the quotient plan over a dividend containing `i128::MIN` and a divisor containing `-1` under each profile, with no result interval and with `[-1, 1]`, and assert `CapacityExceeded { operation: Divide }`. Construct the remainder and `Modulo` plans over the same intervals, a dividend `[i128::MIN, i128::MIN]` with divisor `[1, 2]`, and a dividend `[i128::MIN + 1, 0]` with divisor `[-1, -1]`. Compare each capacity with independently written literals, including divisor `[i128::MIN, 5]` (remainder capacity `[-i128::MAX, i128::MAX]`) and divisor `[0, 0]` (capacity `[0, 0]`).
3. For AC-3, evaluate every grid case through the plan, through `divide` or `modulo` over `Integer`, and through `evaluate_native_integer_division` or `evaluate_native_integer_modulo`. Each runs on a fresh recording `Meter` at a sufficient limit and at one below each charge. Compare the outcome kind, the value, the refusal payload (`DivisionOutOfDomain { member }`, `ModuloOutOfDomain`, `Undefined(DivisionByZero)`), every charge event and the counters. Check against independent literals: `-7 div 2` is `-3`, `-4`, `-4`; `-7 div -2` is `3`, `3`, `4`; `-7 mod 2` and `-7 mod -2` are `1`; `i64::MIN div -1` is `2^63`; `i64::MIN mod -1` is `0`. Grid pairs whose quotient plan is refused by step 2 are not evaluated as quotients.
4. For AC-4, run `charge_native_integer_schedule` over each admitted `Divide` and `Modulo` operation and over `Division` and `Modulus` with the same operands, including a zero divisor, and compare events and counters. Re-run the existing native arithmetic and FR-397 tests unchanged.
5. For AC-5 and AC-6, list the harnesses of FR-409 Behavior 13 and 14 and check that each is present: three plan harnesses, and for each of the seven member combinations five symbolic-width harnesses and six constant-divisor harnesses (77 evaluation harnesses). Check that each evaluation harness draws its result interval as `None` or a sorted pair of unrestricted endpoints, that each remainder and `Modulo` harness takes its companion quotient from the plan-free `evaluate_native_integer_division` and not from the plan under test, and that each harness names its assigned harness mutant per FR-409 Behavior 22's family table. Code review inspects each expected side for a shared function with the production body, for `/`, `%`, `checked_div`, `checked_rem`, `wrapping_div`, `wrapping_rem`, `div_euclid` and `rem_euclid`, for quotient-then-divisor operand order and for a docstring of the outputs it determines. It inspects plan construction and admission for division of a symbolic operand and for `Integer` construction. There is no automated step: a source-text scan is not a test.
6. For AC-7, run the probe first: the truncating quotient harness at `w` = 8 and at `w` = 64 (symbolic divisor), each under `timeout 600`. Then run every harness one job at a time, each under a 600-second limit, inside a memory-limited scope that records its peak, with the cgroup `memory.peak` or an equal measured source as provenance. Record outcome, time, peak with provenance and harness id. Within a member combination, run the symbolic widths in ascending order. After the first width that is a measured row, do not run the larger widths: record each as not run and covered by that row. For each VERIFIED harness, confirm every cover is reached and run its assigned harness mutant to a FAILED verdict.
7. For AC-8, record each measured row in the evidence record below, naming operation kind, profile, member, divisor shape and smallest failing width, and check that the kernel holds no other copy. CG copies the rows from here into CG FR-056's table; that copy is CG's step, not this procedure's. Code review searches the kernel for a refusal variant, refusal path or `Meter` behaviour defined for a measured row; none may exist.
8. For AC-9, run each mutant through steps 2 and 3 and assert it fails at the step named. Then, except for the moved zero guard (host-only: it changes only charge events, which no harness observes), run it under Kani in the harness named, assert FAILED and extract the concrete playback. For an evaluation harness, replay the decoded operands through `divide` or `modulo` over `Integer` and assert that the kernel outcome differs from the mutant's. For a plan harness, replay the decoded intervals through the healthy `NativeIntegerPlan::new` and assert that it refuses, or derives a different capacity, where the mutant constructed.
9. For AC-10, inspect the Status of FR-409 and of this TC for a verdict or table row per member combination, shape and width, and for no claim without a record.
10. For AC-11, run `make verus` and record the result in the Verus record below.
11. For AC-12, run `scripts/ir1556_verus_mutants.py` and confirm every mutant is reported killed by a failed proof obligation and none as UNRELIABLE.

### Mutants

| Mutant (site) | Distinguishing input | Fails at | Kani harness |
| --- | --- | --- | --- |
| Truncating plan evaluated as floor | `-7 div 2` (quotient `-3` versus `-4`) | step 3 | truncating quotient, `w` = 8 |
| Truncating plan evaluated as Euclidean | `-7 div 2` | step 3 | truncating quotient, `w` = 8 |
| Floor plan evaluated as truncating | `-7 div 2` | step 3 | floor quotient, `w` = 8 |
| Floor plan evaluated as Euclidean | `-7 div -2` (quotient `3` versus `4`) | step 3 | floor quotient, `w` = 8 |
| Euclidean plan evaluated as truncating | `-7 div 2` | step 3 | Euclidean quotient, `w` = 8 |
| Euclidean plan evaluated as floor | `-7 div -2` | step 3 | Euclidean quotient, `w` = 8 |
| `Modulo` evaluated as the truncating remainder | `-7 mod 2` (`1` versus `-1`) | step 3 | `Modulo`, `w` = 8 |
| `Modulo` evaluated as the floor remainder | `-7 mod -2` (`1` versus `-1`) | step 3 | `Modulo`, `w` = 8 |
| Zero-divisor check removed | any dividend over divisor `0` (`Undefined(DivisionByZero)` versus a panic or completion) | step 3 | each profile's quotient, `w` = 8 |
| Zero-divisor check moved after the arithmetic charge | `5 div 0` at a sufficient limit (an extra charge event) | step 3 | none (charges are host-only) |
| MIN / -1 capacity refusal removed | quotient plan over dividend `[i128::MIN, 0]` and divisor `[-1, -1]` | step 2 | quotient plan harness |
| Truncating remainder evaluated as floor, or as Euclidean | `-7 rem 2` (`-1` versus `1`) | step 3 | truncating remainder, `w` = 8 |
| Floor remainder evaluated as truncating | `-7 rem 2` | step 3 | floor remainder, `w` = 8 |
| Floor remainder evaluated as Euclidean | `-7 rem -2` (`-1` versus `1`) | step 3 | floor remainder, `w` = 8 |
| Euclidean remainder evaluated as truncating | `-7 rem 2` | step 3 | Euclidean remainder, `w` = 8 |
| Euclidean remainder evaluated as floor | `-7 rem -2` | step 3 | Euclidean remainder, `w` = 8 |
| Remainder member returns the quotient | `7 rem 2` under truncating | step 3 | truncating remainder, `w` = 8 |
| Result-interval check skipped | `7 div 2` with result interval `[0, 2]`; in a constant `-1` shape, `5 div -1` with result interval `[0, 0]` and `5 rem -1` with result interval `[1, 1]` | step 3 | truncating quotient, `w` = 8; every constant `-1` harness (its assigned mutant) |
| Remainder capacity widened to `[-B', B']` | remainder plan over divisor `[2, 3]` (capacity `[-2, 2]` versus `[-3, 3]`) | step 2 | remainder plan harness |
| `Modulo` capacity lower endpoint lowered to `-1` | `Modulo` plan over divisor `[2, 3]` (capacity `[0, 2]` versus `[-1, 2]`) | step 2 | `Modulo` plan harness |

## Evidence record

The versioned record of every executed harness, filled by the coder from executed runs. It is the kernel's only record of a measured row. CG copies each measured row from here into CG FR-056's tractability table, which IR-1526's seam loads and which is the single shipped record. Each row gives the harness id, operation kind, profile, member, divisor shape, width, outcome (VERIFIED, or timeout, memory exhaustion or unreliable; a FAILED verdict is a defect, not a row), time and peak memory with provenance. A symbolic width above its combination's smallest failing width is listed with the outcome `not run, covered by <harness id>`.

Every harness ran once under `cargo kani` with a 600-second wall-clock limit, one job at a time, inside a 24 GiB memory-limited cgroup scope. Time is wall-clock seconds. Peak is the cgroup `memory.peak` read inside the scope (it includes the build); every peak of a harness run is below the 16 GiB ceiling. A `timeout (600 s)` row is a measured row, not a proof. A `not run` row is covered by the smaller-width row of its member combination. "Required covers reached" counts only the covers that belong to that harness's operation and shape (FR-409 Behavior 16); Kani's own cover total also counts covers of other combinations that share the harness body, so it is not used. The last column is the assigned harness mutant of Behavior 22: its Kani verdict on that harness, then the verdict of replaying the decoded playback on the healthy kernel and on the mutant (`REPLAY-DIFFERS`: the healthy plan equals the exact oracle and the mutant differs).

The Behavior 17 probe ran first (truncating quotient, symbolic divisor, under a 10-minute limit). Its two records:

| Harness id | Kind | Profile | Member | Shape | Width | Outcome | Time (s) | Peak (GiB, cgroup memory.peak) | Required covers reached | Assigned mutant |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| probe trunc_quotient_sym_w8 | Divide | Truncating | Quotient | symbolic | 8 | VERIFIED | 47.8 | 1.67 | 3/3 required | - |
| probe trunc_quotient_sym_w64 | Divide | Truncating | Quotient | symbolic | 64 | timeout (600 s) | 600.0 | 1.67 | not reached | - |

The ladder, in full. The probe's width-64 run is the record of `trunc_quotient_sym_w64`; the ladder did not repeat it.

| Harness id | Kind | Profile | Member | Shape | Width | Outcome | Time (s) | Peak (GiB, cgroup memory.peak) | Required covers reached | Assigned mutant |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| divide_quotient_plan_refuses_min_over_minus_one_and_derives_capacity | Divide | all three | Quotient | plan | - | VERIFIED | 3.3 | 0.45 | 4/4 required | min-over-minus-one-unguarded FAILED REPLAY-DIFFERS |
| divide_remainder_plan_derives_capacity_and_never_refuses | Divide | all three | Remainder | plan | - | VERIFIED | 3.7 | 0.43 | 3/3 required | remainder-capacity-widened FAILED REPLAY-DIFFERS |
| modulo_plan_derives_capacity_and_never_refuses | Modulo | all three | - | plan | - | VERIFIED | 3.0 | 0.43 | 3/3 required | modulo-capacity-lowered FAILED REPLAY-DIFFERS |
| trunc_quotient_sym_w8 | Divide | Truncating | Quotient | symbolic | 8 | VERIFIED | 48.6 | 1.67 | 3/3 required | swap-truncating-floor FAILED REPLAY-DIFFERS; swap-truncating-euclidean FAILED REPLAY-DIFFERS; zero-guard-removed FAILED REPLAY-DIFFERS; result-interval-skipped FAILED REPLAY-DIFFERS |
| trunc_quotient_sym_w16 | Divide | Truncating | Quotient | symbolic | 16 | VERIFIED | 374.7 | 1.67 | 3/3 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_quotient_sym_w32 | Divide | Truncating | Quotient | symbolic | 32 | timeout (600 s) | 600.0 | 1.67 | not reached | - |
| trunc_quotient_sym_w64 | Divide | Truncating | Quotient | symbolic | 64 | timeout (600 s) | 600.0 | 1.67 | not reached | - |
| trunc_quotient_sym_w128 | Divide | Truncating | Quotient | symbolic | 128 | not run, covered by trunc_quotient_sym_w32 | | | | |
| floor_quotient_sym_w8 | Divide | Floor | Quotient | symbolic | 8 | VERIFIED | 51.5 | 1.71 | 3/3 required | swap-floor-truncating FAILED REPLAY-DIFFERS; swap-floor-euclidean FAILED REPLAY-DIFFERS; zero-guard-removed FAILED REPLAY-DIFFERS |
| floor_quotient_sym_w16 | Divide | Floor | Quotient | symbolic | 16 | timeout (600 s) | 600.0 | 1.71 | not reached | - |
| floor_quotient_sym_w32 | Divide | Floor | Quotient | symbolic | 32 | not run, covered by floor_quotient_sym_w16 | | | | |
| floor_quotient_sym_w64 | Divide | Floor | Quotient | symbolic | 64 | not run, covered by floor_quotient_sym_w16 | | | | |
| floor_quotient_sym_w128 | Divide | Floor | Quotient | symbolic | 128 | not run, covered by floor_quotient_sym_w16 | | | | |
| euclid_quotient_sym_w8 | Divide | Euclidean | Quotient | symbolic | 8 | VERIFIED | 52.3 | 1.71 | 3/3 required | swap-euclidean-truncating FAILED REPLAY-DIFFERS; swap-euclidean-floor FAILED REPLAY-DIFFERS; zero-guard-removed FAILED REPLAY-DIFFERS |
| euclid_quotient_sym_w16 | Divide | Euclidean | Quotient | symbolic | 16 | timeout (600 s) | 600.0 | 1.71 | not reached | - |
| euclid_quotient_sym_w32 | Divide | Euclidean | Quotient | symbolic | 32 | not run, covered by euclid_quotient_sym_w16 | | | | |
| euclid_quotient_sym_w64 | Divide | Euclidean | Quotient | symbolic | 64 | not run, covered by euclid_quotient_sym_w16 | | | | |
| euclid_quotient_sym_w128 | Divide | Euclidean | Quotient | symbolic | 128 | not run, covered by euclid_quotient_sym_w16 | | | | |
| trunc_remainder_sym_w8 | Divide | Truncating | Remainder | symbolic | 8 | VERIFIED | 66.4 | 2.23 | 3/3 required | swap-truncating-floor FAILED REPLAY-DIFFERS; swap-truncating-euclidean FAILED REPLAY-DIFFERS; remainder-returns-quotient FAILED REPLAY-DIFFERS |
| trunc_remainder_sym_w16 | Divide | Truncating | Remainder | symbolic | 16 | VERIFIED | 437.8 | 2.24 | 3/3 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_remainder_sym_w32 | Divide | Truncating | Remainder | symbolic | 32 | timeout (600 s) | 600.0 | 2.24 | not reached | - |
| trunc_remainder_sym_w64 | Divide | Truncating | Remainder | symbolic | 64 | not run, covered by trunc_remainder_sym_w32 | | | | |
| trunc_remainder_sym_w128 | Divide | Truncating | Remainder | symbolic | 128 | not run, covered by trunc_remainder_sym_w32 | | | | |
| floor_remainder_sym_w8 | Divide | Floor | Remainder | symbolic | 8 | VERIFIED | 73.6 | 2.28 | 3/3 required | swap-floor-truncating FAILED REPLAY-DIFFERS; swap-floor-euclidean FAILED REPLAY-DIFFERS |
| floor_remainder_sym_w16 | Divide | Floor | Remainder | symbolic | 16 | timeout (600 s) | 600.0 | 2.27 | not reached | - |
| floor_remainder_sym_w32 | Divide | Floor | Remainder | symbolic | 32 | not run, covered by floor_remainder_sym_w16 | | | | |
| floor_remainder_sym_w64 | Divide | Floor | Remainder | symbolic | 64 | not run, covered by floor_remainder_sym_w16 | | | | |
| floor_remainder_sym_w128 | Divide | Floor | Remainder | symbolic | 128 | not run, covered by floor_remainder_sym_w16 | | | | |
| euclid_remainder_sym_w8 | Divide | Euclidean | Remainder | symbolic | 8 | VERIFIED | 88.2 | 2.28 | 3/3 required | swap-euclidean-truncating FAILED REPLAY-DIFFERS; swap-euclidean-floor FAILED REPLAY-DIFFERS |
| euclid_remainder_sym_w16 | Divide | Euclidean | Remainder | symbolic | 16 | timeout (600 s) | 600.0 | 2.27 | not reached | - |
| euclid_remainder_sym_w32 | Divide | Euclidean | Remainder | symbolic | 32 | not run, covered by euclid_remainder_sym_w16 | | | | |
| euclid_remainder_sym_w64 | Divide | Euclidean | Remainder | symbolic | 64 | not run, covered by euclid_remainder_sym_w16 | | | | |
| euclid_remainder_sym_w128 | Divide | Euclidean | Remainder | symbolic | 128 | not run, covered by euclid_remainder_sym_w16 | | | | |
| modulo_sym_w8 | Modulo | Euclidean (Modulo) | - | symbolic | 8 | VERIFIED | 64.7 | 2.26 | 3/3 required | modulo-truncating FAILED REPLAY-DIFFERS; modulo-floor FAILED REPLAY-DIFFERS |
| modulo_sym_w16 | Modulo | Euclidean (Modulo) | - | symbolic | 16 | timeout (600 s) | 600.0 | 2.26 | not reached | - |
| modulo_sym_w32 | Modulo | Euclidean (Modulo) | - | symbolic | 32 | not run, covered by modulo_sym_w16 | | | | |
| modulo_sym_w64 | Modulo | Euclidean (Modulo) | - | symbolic | 64 | not run, covered by modulo_sym_w16 | | | | |
| modulo_sym_w128 | Modulo | Euclidean (Modulo) | - | symbolic | 128 | not run, covered by modulo_sym_w16 | | | | |
| trunc_quotient_const_7_w64 | Divide | Truncating | Quotient | constant 7 | 64 | VERIFIED | 121.8 | 1.67 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_quotient_const_7_w128 | Divide | Truncating | Quotient | constant 7 | 128 | VERIFIED | 527.0 | 1.60 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_quotient_const_neg7_w64 | Divide | Truncating | Quotient | constant -7 | 64 | VERIFIED | 93.0 | 1.67 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_quotient_const_neg7_w128 | Divide | Truncating | Quotient | constant -7 | 128 | VERIFIED | 496.9 | 1.79 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_quotient_const_neg1_w64 | Divide | Truncating | Quotient | constant -1 | 64 | VERIFIED | 37.3 | 1.67 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| trunc_quotient_const_neg1_w128 | Divide | Truncating | Quotient | constant -1 | 128 | VERIFIED | 38.9 | 1.75 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| floor_quotient_const_7_w64 | Divide | Floor | Quotient | constant 7 | 64 | VERIFIED | 343.4 | 1.71 | 2/2 required | swap-floor-truncating FAILED REPLAY-DIFFERS |
| floor_quotient_const_7_w128 | Divide | Floor | Quotient | constant 7 | 128 | timeout (600 s) | 600.0 | 1.64 | not reached | - |
| floor_quotient_const_neg7_w64 | Divide | Floor | Quotient | constant -7 | 64 | VERIFIED | 469.4 | 1.71 | 2/2 required | swap-floor-truncating FAILED REPLAY-DIFFERS |
| floor_quotient_const_neg7_w128 | Divide | Floor | Quotient | constant -7 | 128 | timeout (600 s) | 600.0 | 1.64 | not reached | - |
| floor_quotient_const_neg1_w64 | Divide | Floor | Quotient | constant -1 | 64 | VERIFIED | 33.4 | 1.71 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| floor_quotient_const_neg1_w128 | Divide | Floor | Quotient | constant -1 | 128 | VERIFIED | 44.9 | 1.81 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| euclid_quotient_const_7_w64 | Divide | Euclidean | Quotient | constant 7 | 64 | VERIFIED | 331.0 | 1.71 | 2/2 required | swap-euclidean-truncating FAILED REPLAY-DIFFERS |
| euclid_quotient_const_7_w128 | Divide | Euclidean | Quotient | constant 7 | 128 | timeout (600 s) | 600.0 | 1.63 | not reached | - |
| euclid_quotient_const_neg7_w64 | Divide | Euclidean | Quotient | constant -7 | 64 | timeout (600 s) | 600.0 | 1.71 | not reached | - |
| euclid_quotient_const_neg7_w128 | Divide | Euclidean | Quotient | constant -7 | 128 | timeout (600 s) | 600.0 | 1.64 | not reached | - |
| euclid_quotient_const_neg1_w64 | Divide | Euclidean | Quotient | constant -1 | 64 | VERIFIED | 35.6 | 1.71 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| euclid_quotient_const_neg1_w128 | Divide | Euclidean | Quotient | constant -1 | 128 | VERIFIED | 38.7 | 1.80 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| trunc_remainder_const_7_w64 | Divide | Truncating | Remainder | constant 7 | 64 | VERIFIED | 80.6 | 2.24 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_remainder_const_7_w128 | Divide | Truncating | Remainder | constant 7 | 128 | VERIFIED | 398.4 | 2.05 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_remainder_const_neg7_w64 | Divide | Truncating | Remainder | constant -7 | 64 | VERIFIED | 71.3 | 2.24 | 2/2 required | swap-truncating-floor FAILED REPLAY-DIFFERS |
| trunc_remainder_const_neg7_w128 | Divide | Truncating | Remainder | constant -7 | 128 | timeout (600 s) | 600.0 | 2.05 | not reached | - |
| trunc_remainder_const_neg1_w64 | Divide | Truncating | Remainder | constant -1 | 64 | VERIFIED | 52.8 | 1.98 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| trunc_remainder_const_neg1_w128 | Divide | Truncating | Remainder | constant -1 | 128 | VERIFIED | 39.4 | 2.05 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| floor_remainder_const_7_w64 | Divide | Floor | Remainder | constant 7 | 64 | VERIFIED | 114.0 | 2.24 | 2/2 required | swap-floor-truncating FAILED REPLAY-DIFFERS |
| floor_remainder_const_7_w128 | Divide | Floor | Remainder | constant 7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| floor_remainder_const_neg7_w64 | Divide | Floor | Remainder | constant -7 | 64 | VERIFIED | 129.7 | 2.27 | 2/2 required | swap-floor-truncating FAILED REPLAY-DIFFERS |
| floor_remainder_const_neg7_w128 | Divide | Floor | Remainder | constant -7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| floor_remainder_const_neg1_w64 | Divide | Floor | Remainder | constant -1 | 64 | VERIFIED | 45.5 | 2.23 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| floor_remainder_const_neg1_w128 | Divide | Floor | Remainder | constant -1 | 128 | VERIFIED | 39.4 | 2.09 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| euclid_remainder_const_7_w64 | Divide | Euclidean | Remainder | constant 7 | 64 | VERIFIED | 147.5 | 2.23 | 2/2 required | swap-euclidean-truncating FAILED REPLAY-DIFFERS |
| euclid_remainder_const_7_w128 | Divide | Euclidean | Remainder | constant 7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| euclid_remainder_const_neg7_w64 | Divide | Euclidean | Remainder | constant -7 | 64 | VERIFIED | 114.8 | 2.27 | 2/2 required | swap-euclidean-truncating FAILED REPLAY-DIFFERS |
| euclid_remainder_const_neg7_w128 | Divide | Euclidean | Remainder | constant -7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| euclid_remainder_const_neg1_w64 | Divide | Euclidean | Remainder | constant -1 | 64 | VERIFIED | 40.0 | 2.25 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| euclid_remainder_const_neg1_w128 | Divide | Euclidean | Remainder | constant -1 | 128 | VERIFIED | 44.6 | 2.08 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| modulo_const_7_w64 | Modulo | Euclidean (Modulo) | - | constant 7 | 64 | VERIFIED | 116.0 | 2.23 | 2/2 required | modulo-truncating FAILED REPLAY-DIFFERS |
| modulo_const_7_w128 | Modulo | Euclidean (Modulo) | - | constant 7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| modulo_const_neg7_w64 | Modulo | Euclidean (Modulo) | - | constant -7 | 64 | VERIFIED | 103.9 | 2.27 | 2/2 required | modulo-truncating FAILED REPLAY-DIFFERS |
| modulo_const_neg7_w128 | Modulo | Euclidean (Modulo) | - | constant -7 | 128 | timeout (600 s) | 600.0 | 2.08 | not reached | - |
| modulo_const_neg1_w64 | Modulo | Euclidean (Modulo) | - | constant -1 | 64 | VERIFIED | 38.3 | 2.01 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |
| modulo_const_neg1_w128 | Modulo | Euclidean (Modulo) | - | constant -1 | 128 | VERIFIED | 47.1 | 2.08 | 2/2 required | result-interval-skipped FAILED REPLAY-DIFFERS |

### Verus record

Run by `scripts/verus.sh` (`make verus`) on the source of this change. The proof is `src/native_integer/verus_proofs.rs` (FR-409 Behavior 24).

| Run | Outcome | Time |
| --- | --- | --- |
| `make verus` (the whole crate) | `221 verified, 0 errors` | 4.6 s wall |
| `scripts/ir1556_verus_mutants.py` | unmutated copy verified; 10 of 10 mutants killed by a failed postcondition, none UNRELIABLE | 37 s wall for the 11 runs |

The theorem is the specification of `native_div_rem`, `div_rem_post`: for every `i128` dividend `a` and nonzero divisor `b`, under each profile, the remainder `r` is in the profile's window, a quotient `q` satisfies `a = q b + r`, and the quotient is absent exactly for `i128::MIN / -1`, with remainder zero. It rests on `lemma_truncating`, the facts of Rust's truncating `/` and `%`. This is the relation of every timeout row of the table above (`a - q b` in the window), for every width at once and for symbolic and constant divisors alike. It does not cover the choice of the member, the result interval or the outcome kinds, which the VERIFIED rows and the differential cover.

Mutants (`scripts/ir1556_verus_mutants.py`). A mutant is killed only by a failed proof obligation; a run that only exhausts the solver's resource limit would be reported as UNRELIABLE and fail the script. Each of these fails the postcondition of `native_div_rem`: the truncating return not taken on a zero remainder; the floor step taken on equal signs; the floor step decided by the dividend's sign; the floor quotient not stepped; the floor quotient stepped up; the floor remainder not shifted; the floor pair returned for the Euclidean profile; the Euclidean quotient not stepped; the Euclidean remainder shifted the wrong way; a quotient kept for `i128::MIN / -1`.

## Expected Results

Every grid case through the plan has the outcome, value, refusal payload and charge events of `divide` or `modulo` over `Integer` and of the plan-free evaluator. The quotient plan at `i128::MIN` with `-1` is a capacity refusal. Every mutant fails at the step named. Each mutant with a Kani harness yields a FAILED verdict whose playback, replayed on the host, differs from the healthy kernel. Each harness ends in one of three states: an executed VERIFIED verdict with its covers reached and its assigned mutant killed; a measured row with its time and peak; or, for a symbolic width above the smallest failing width, not run and covered by that row. No harness is left without a record.

## Status

Run under IR-1541. The `Divide` and `Modulo` plan variants exist. The host differential (`tests/ir1541_native_integer_divide.rs`) passes and all 15 host mutants of Behavior 22 are killed. The Behavior 17 probe ran first: truncating quotient at width 8 VERIFIED (47.8 s, 1.67 GiB) and at width 64 timeout (600 s, 1.67 GiB). All 80 Kani harnesses of the ladder reached a terminal state, none FAILED. 42 are VERIFIED with every cover that belongs to the harness reached: the three plan harnesses; the symbolic divisor at width 8 for all seven member combinations; the symbolic divisor at width 16 for truncating quotient and truncating remainder only; every constant `-1` harness at widths 64 and 128; and the constant `7` and `-7` harnesses at width 64 for every combination except Euclidean quotient `-7` (timeout), plus width 128 for truncating quotient (`7`, `-7`) and truncating remainder (`7`). 20 are measured rows with outcome timeout at 600 s: the symbolic divisor at width 16 for floor quotient, Euclidean quotient, floor remainder, Euclidean remainder and `Modulo`, at width 32 for truncating quotient and truncating remainder, at width 64 for truncating quotient (the probe's record), the constant `7`/`-7` harnesses at width 128 not listed as VERIFIED, and Euclidean quotient `-7` at width 64. 18 further symbolic-divisor harnesses are recorded as not run, covered by the smaller-width row of their combination. No harness hit memory exhaustion, and every harness peak is below the 16 GiB ceiling. Each of the 42 VERIFIED harnesses had its assigned mutant run on Kani (54 runs, because some harnesses share a mutant or take two); every run returned FAILED, and every decoded playback, replayed on the healthy kernel and on the mutant from the kept Kani logs, shows the healthy plan equal to the exact oracle and the mutant differing (54 of 54 REPLAY-DIFFERS). An earlier recording showed 23 REPLAY-SAME rows; the cause was the replay script sharing one build directory between the healthy kernel and the mutant copies, so the healthy replay reused the previous mutant's test binary. The script now builds each copy in its own directory and all replays were re-run. FR-409-AC-7 and FR-409-AC-9 are checked by `the_evidence_record_settles_every_harness_with_its_probe_and_mutant`, which reads the evidence record of TC-957; the Kani runs and the playback replays themselves are executed by `scripts/ir1541_ladder.py` and `scripts/ir1541_native_integer_mutants.py` (the replay driver is an ignored test, `replay_a_decoded_playback`, run by that script). A proof is claimed only for a VERIFIED harness; no proof is claimed for a timeout row or for a combination as a whole, and the evidence record of TC-957 is the only copy of the measured rows.

Steps 10 and 11 (AC-11 and AC-12, IR-1556): the Verus record above. The full-width relation is proved on `native_div_rem`; the 20 timeout rows stay recorded as measured rows under Kani.
