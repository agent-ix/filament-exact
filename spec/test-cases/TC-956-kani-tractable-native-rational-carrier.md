---
id: TC-956
title: "The native Rational carrier matches the owner route and verifies"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-408
    type: verifies
---
# TC-956: The native Rational carrier matches the owner route and verifies

## Description

Verify FR-408-AC-1 through FR-408-AC-12 through the public kernel API, a literal-charge test, the owner differential and the recorded Kani measurement. The differential lives in filament-exact's `tests/`, because the owner route (`Rational`, `evaluate_rational_arithmetic`, `order_numbers`, `planned_equality`) is in the same crate. Each mutant names the input or inspection that fails it.

## Test Procedure

1. For AC-1, check `Copy` by a generic bound, no drop glue by `needs_drop`, zero allocations of construction, accessors and every operation in a counting allocator window, and that no field is public (a struct-literal program fails with `E0451`).
2. For AC-2, call `from_i64_parts` over the admitted, `ZeroDenominator` and `NonCanonical` pairs of the criterion with literals written independently of the carrier, with a `Meter`-free signature (coerce to a function pointer), and check the precedence with `(0, 0)`.
3. For AC-3, round-trip the CG domain (111 canonical values) and the boundary set through `from_rational` and `to_rational` and compare with `Rational::new`; present `2^127/1` and `1/2^127`.
4. For AC-4 and AC-5, evaluate the literal cases of the criteria and the four orderings and equality over every ordered pair of `{-1/2, 0/1, 1/3, 1/2, 2/3}` against a table written independently.
5. For AC-6, present the capacity operands of the criterion to each operation, with a recording `Meter` at a zero budget, and assert the named quantity and the unchanged `Meter`; assert that no pair of `i64` parts is a capacity refusal for any operation over a boundary sample and a random sample.
6. For AC-7 and AC-8, run the literal charge cases (including `9/1 + 9/1`, operands 4, arithmetic 6) under a recording `Meter` at a sufficient, an exact and a one-less limit for each limit kind used and compare the charge points, limit kinds, amounts, denied charge, counters and incomplete record with literals written independently of the carrier; run the `a + b`, `b + a`, `=` sequence on one `Meter`, with the denial placed in the second addition.
7. For AC-9, run the differential of FR-408 Behavior 19 beside the owner route over the 12,321 pairs of the CG domain for each of the seven operations and over the boundary set, at the three limits, comparing outcome, value, Boolean, every charge, the denied charge, counters and the incomplete record; list the capacity inputs and check each.
8. AC-10 is a code-review inspection item (FR-408 Behavior 15 and the one-shared-module rule); it has no automated step. The one-source property is evidenced functionally by step 9 running the shipped function.
9. For AC-11, execute the harnesses of FR-408 Behavior 18 under the authorized Kani runner (`timeout 900`) and record the outcome, time and peak memory in the evidence record below; confirm every cover of a verified harness is reached and each mutant fails. The Kani arithmetic harnesses check the aggregate `integer_bits` high-water, the counters and the admission count; the `rational-arithmetic.normalize` amount on its own is owned by steps 6 and 7, because it never exceeds the arithmetic amount.
10. Run each mutant through the step it names.
11. For AC-12, run the host sweep over every numerator in `[-600, 600]` and denominator in `[1, 600]` (the tier covers up to 255; the rest take the `u64` tier), run the two tier harnesses of the evidence record, and run `scripts/ir258_u8_tier_mutants.py` (host mutants of the tier) and `scripts/ir258_u8_tier_mutants.py --kani-rows` (the arithmetic mutants against the Kani rows).

### Mutants

| Mutant | Fails at |
| --- | --- |
| Add forms `a q - c p` | `1/2 + 1/3` of step 4 |
| Subtract has its operands swapped | `-1/2 - 1/3` of step 4 |
| The result is not reduced | `1/2 + 1/2` of step 4 and the differential |
| Reduction runs fewer than 256 gcd steps (for example 128) | the 252-step pair `(2^126 - 1)/2^126` times `1/1` in the differential boundary set of step 7 |
| The sign is put on the denominator instead of the numerator | `-1/2 - 1/3` of step 4 |
| Zero is retained as `0/d` | `1/2 - 1/2` of step 4 |
| `from_i64_parts` normalizes instead of refusing | `(2, 4)` of step 2 |
| `NonCanonical` is judged after `ZeroDenominator` is skipped | `(0, 0)` of step 2 |
| `(0, 2)` is admitted | step 2 |
| Equality compares the cross product only for the numerators | `1/2` against `-1/2` of step 4 |
| `LessOrEqual` is evaluated as `Less` | the equal pair of step 4 and the differential |
| Ordering compares `a p` with `c q` | `1/3 < 1/2` of step 4 |
| The sum wraps instead of refusing | the `Combination` case of step 5 |
| Subtract negates `c` and refuses on `i128::MIN` | `-1/1 - i128::MIN/1` of step 5 |
| A capacity refusal charges the `Meter` | step 5 |
| The normalize charge uses the reduced parts | `1/2 + 1/2` of step 6 and the differential (not a Kani row; see step 9) |
| The arithmetic charge omits the `+ 1` of add | `9/1 + 9/1` of step 6 (arithmetic `integer_bits` 6, not 5) and the differential |
| `result-retain` is charged before `normalize` | step 6 and the differential |
| A denied charge consumes a result unit | the one-less limits of step 6 |
| Equality charges the ordering schedule | step 6 |
| The carrier owns a `Rational` | step 1 |

## Evidence record

`timeout 900` per harness, source `src/native_rational/kani_proofs.rs` of this change, one job at a time under `flock /tmp/cargo-slot.lock` and `systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0`. "Time" is Kani's reported verification time followed by the wall time of the whole `cargo kani` invocation; TIMED OUT means no terminal verdict at 900 s, is the measured row and claims no proof. Peak memory is the scope's cgroup `memory.peak`, sampled while the run was alive; the scope held the whole invocation, so it includes the build and is an upper bound on the solver's peak. Every row is a fresh run on the kernel with the `u8` tier.

| Harness id | Outcome | Time | Peak memory |
| --- | --- | --- | --- |
| `native_rational_from_i64_parts_matches_reference` (a, full `i64`) | TIMED OUT | 900 s (limit; killed, no verdict) | 2147209216 bytes (2.00 GiB) |
| `native_rational_equal_cg_domain_matches_reference` | VERIFIED, covers reached (2 of 2) | 35.7 s (wall 41.2 s) | 1460588544 bytes (1.36 GiB) |
| `native_rational_order_cg_domain_matches_reference` | VERIFIED, covers reached (2 of 2) | 59.2 s (wall 67.4 s) | 1909317632 bytes (1.78 GiB) |
| `native_rational_multiply_cg_domain_matches_reference` | VERIFIED, covers reached (3 of 3) | 128.9 s (wall 132.2 s) | 3492552704 bytes (3.25 GiB) |
| `native_rational_add_cg_domain_matches_reference` | VERIFIED, covers reached (3 of 3) | 277.7 s (wall 280.7 s) | 3560067072 bytes (3.32 GiB) |
| `native_rational_subtract_cg_domain_matches_reference` | VERIFIED, covers reached (3 of 3) | 297.4 s (wall 300.4 s) | 3591430144 bytes (3.34 GiB) |
| `native_rational_subtract_cg_domain_from_i64_parts_matches_reference` | VERIFIED, covers reached (3 of 3) | 327.3 s (wall 330.2 s) | 4093837312 bytes (3.81 GiB) |
| `native_rational_add_is_commutative_over_cg_domain` (c) | VERIFIED, covers reached (1 of 1) | 342.6 s (wall 348.8 s) | 6649708544 bytes (6.19 GiB) |
| `native_rational_add_is_commutative_from_i64_parts_over_cg_domain` (c, carriers by `from_i64_parts`) | VERIFIED, covers reached (1 of 1) | 345.8 s (wall 348.4 s) | 6830206976 bytes (6.36 GiB) |
| `native_fraction_gcd8_matches_euclid_over_u8` (tier, whole `u8` range) | VERIFIED, covers reached (2 of 2) | 9.2 s (wall 11.2 s) | 420405248 bytes (0.39 GiB) |
| `native_fraction_reduce_u8_tier_matches_euclid` (tier through `reduce`, magnitudes up to 255) | VERIFIED, covers reached (1 of 1) | 337.2 s (wall 339.3 s) | 2018938880 bytes (1.88 GiB) |
| `native_rational_add_denies_the_operands_occurrences` | VERIFIED, covers reached (1 of 1) | 27.7 s (wall 30.8 s) | 1561759744 bytes (1.45 GiB) |
| `native_rational_add_denies_the_arithmetic_bits_one_short` | VERIFIED, covers reached (1 of 1) | 96.6 s (wall 98.2 s) | 4231692288 bytes (3.94 GiB) |
| `native_rational_add_denies_the_fourth_work_unit` | VERIFIED, covers reached (1 of 1) | 56.9 s (wall 60.3 s) | 3602132992 bytes (3.35 GiB) |
| `native_rational_add_denies_the_result_unit` | VERIFIED, covers reached (1 of 1) | 63.9 s (wall 67.2 s) | 3603984384 bytes (3.36 GiB) |
| `native_rational_subtract_denies_the_operands_occurrences` | VERIFIED, covers reached (1 of 1) | 4.2 s (wall 7.3 s) | 706048000 bytes (0.66 GiB) |
| `native_rational_subtract_denies_the_result_unit` | VERIFIED, covers reached (1 of 1) | 22.0 s (wall 25.1 s) | 3191726080 bytes (2.97 GiB) |
| `native_rational_multiply_denies_the_operands_occurrences` | VERIFIED, covers reached (1 of 1) | 2.7 s (wall 5.8 s) | 622735360 bytes (0.58 GiB) |
| `native_rational_multiply_denies_the_result_unit` | VERIFIED, covers reached (1 of 1) | 18.4 s (wall 21.6 s) | 3067727872 bytes (2.86 GiB) |
| `native_rational_order_denies_the_operands_occurrences` | VERIFIED, covers reached (1 of 1) | 2.6 s (wall 5.8 s) | 631013376 bytes (0.59 GiB) |
| `native_rational_order_denies_the_result_unit` | VERIFIED, covers reached (1 of 1) | 8.8 s (wall 13.1 s) | 1030311936 bytes (0.96 GiB) |
| `native_rational_equal_denies_the_plan_form_occurrence` | VERIFIED, covers reached (1 of 1) | 2.1 s (wall 5.4 s) | 641679360 bytes (0.60 GiB) |
| `native_rational_equal_denies_the_plan_result_unit` | VERIFIED, covers reached (1 of 1) | 3.1 s (wall 16.7 s) | 647004160 bytes (0.60 GiB) |
| `native_rational_equal_full_i64_matches_reference` | TIMED OUT | 900 s (limit; killed, no verdict) | 1389637632 bytes (1.29 GiB) |
| `native_rational_order_full_i64_matches_reference` | TIMED OUT | 900 s (limit; killed, no verdict) | 1860517888 bytes (1.73 GiB) |
| `native_rational_add_full_i64_matches_reference` | TIMED OUT | 900 s (limit; killed, no verdict) | 15523975168 bytes (14.46 GiB) |
| `native_rational_subtract_full_i64_matches_reference` | TIMED OUT | 900 s (limit; killed, no verdict) | 15575470080 bytes (14.51 GiB) |
| `native_rational_multiply_full_i64_matches_reference` | TIMED OUT | 900 s (limit; killed, no verdict) | 15391862784 bytes (14.33 GiB) |

What the `u8` tier changed. The four CG-domain rows that timed out at 900 s without the tier (add, subtract, subtract with `from_i64_parts` carriers, and the composed predicate (c)) now return VERIFIED with their covers reached, and so does the composed predicate with both operands built by `from_i64_parts`; the multiply CG row, which took 641 s before, now takes 128.9 s. The reference side of the CG-domain arithmetic harnesses is Euclid's algorithm over `u8` (16 steps; both results of the CG domain are below 256, and the reference asserts that range instead of assuming it) in place of 16 steps of 64-bit division, and the carriers are unchanged. The tier is proved equal to Euclid by two harnesses: `native_fraction_gcd8_matches_euclid_over_u8` (the 16-step `u8` gcd against `u8` Euclid for every pair of `u8` values) and `native_fraction_reduce_u8_tier_matches_euclid` (the shipped `reduce`, through all three tiers' selection, against `u8` Euclid for every numerator magnitude up to 255 and every denominator in `[1, 255]`). The rows that were VERIFIED before the tier were rerun on the changed kernel and are VERIFIED again.

The six full-`i64` rows stay measured not-tractable: `from_i64_parts` is a 128-step binary coprime test against 128 steps of Euclid (two gcd algorithms proved equal), equality is the coprimality fact that canonical fractions are equal exactly when they cross-multiply equal, and ordering, add, subtract and multiply are 128-bit products against reference products plus gcd. These are number theory and multiplier problems that a bit-level solver does not close at 900 s; add, subtract and multiply sit at 14.3 to 14.5 GiB, below the 16 GiB ceiling. They wait on the deductive lane (owner: planner-filed Verus lane ticket).

Mutants. The kernel mutants of the tier are in `scripts/ir258_u8_tier_mutants.py`, each an exact-text edit built in a scratch copy with its own target directory. The host sweep (`the_u8_tier_reduces_every_pair_as_euclid_does_across_its_threshold`, FR-408-AC-12) kills all six: the step count cut to 4 and to 6 (7 is the fewest steps that every pair needs, so 6 is the nearest mutant that some input distinguishes), the threshold widened to `u16` while the gcd stays `u8`, the common power of two dropped, the even parts kept, and the difference not reduced to its odd part. The Kani harness `native_fraction_gcd8_matches_euclid_over_u8` fails (VERIFICATION FAILED) on every one of those except the threshold mutant, which lives in `reduce` and is not in the gcd8 harness's reach. The arithmetic mutants fail their CG rows under Kani: addition forming `a q - c p` fails the add row and both composed-predicate rows, subtraction with its operands swapped fails both subtract rows, multiplication forming `a + c` fails the multiply row, and an unreduced result fails the add row.

## Expected Results

Every step passes, each mutant fails at the step it names, and the evidence record holds an executed row for FR-408-AC-11 or the criterion stays planned.

## Status

Steps 1 to 8, 10 and 11's host part run on the host in `tests/ir258_native_rational.rs`. Step 9 is measured in the evidence record: 22 harnesses are VERIFIED with their covers reached and 6 (the full-`i64` group: `from_i64_parts`, equality, ordering, add, subtract, multiply) are TIMED OUT at 900 s with no proof claimed. Of the rows of Behavior 18 (a is full `i64`; b and c are the CG domain), b (add, subtract, multiply, order, equality) and c (the composed predicate) are VERIFIED with their covers reached and mutants that fail them, and a is a measured not-tractable row. AC-11 stays PLANNED: its text names (a) over the full `i64` domain, which is a measured not-tractable row, so AC-11 cannot be fully executed in Kani; b and c are executed. Whether AC-11 closes on "b and c executed, a measured and waiting on the Verus lane" is the planner's ruling. The mutant runs of the equality and ordering CG rows are those of the earlier record and were not rerun on the tiered kernel. AC-12 is met by the host sweep, the two tier harnesses and the mutants above.
