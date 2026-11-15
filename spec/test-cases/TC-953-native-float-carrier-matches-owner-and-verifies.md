---
id: TC-953
title: "The native float carrier matches the owner's float path and verifies under Kani"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: verifies
---
# TC-953: The native float carrier matches the owner's float path and verifies under Kani

## Description

Verify FR-405-AC-1 through FR-405-AC-15 by running the native float operations beside the shipped `evaluate_ieee` and `compare_ieee`, by inspecting the proof-path source, and by recording the Kani probe, verdicts and peaks. Each mutant below is a buildable variant of the named site in a scratch copy and names the input that fails it and the step at which it fails; a mutant that no input distinguishes is not admitted. A timeout, memory exhaustion, missing cover or missing run is inconclusive and never a pass.

## Test Procedure

The boundary set is fixed before the implementation, from independent literals and not from the native body. It holds, for `Binary32` and `Binary64`, in both signs: zero, the smallest subnormal, the largest subnormal, the smallest normal, the largest finite, one, the neighbours of one, infinity, a quiet NaN, a signalling NaN and a NaN with a nonzero payload; operand pairs that are exact ties and one unit off a tie for each rounding mode; pairs that overflow, underflow and cancel exactly; and the operations Negate, Add, Subtract, Multiply, Divide, SquareRoot, FusedMultiplyAdd and the comparisons NumericEqual, TotalOrder, BitIdentical in each of the six rounding spellings (Exact and the five modes). It covers these inputs and not every operand pair.

1. For AC-1 and AC-2, build the carrier from every boundary pattern, check `Copy`, structural equality, the `OutsideCarrierDomain` refusal of an out-of-width pattern with no `Meter` in the signature, and the round trip through `IeeeValue`.
2. For AC-3, call every supported operation and a cross-width call (asserting `ill_typed` and an unchanged `Meter`).
3. For AC-4 and AC-5, evaluate every boundary case natively and by the owner and compare bits, flags, provenance and outcome kind, including the `Exact` refusal's would-be flags with no bits, and compare against independently written literals for the tie, overflow-result, exact-zero-sign and NaN cases.
4. For AC-6, run each case at a sufficient, an exact and a one-less limit natively and by the owner on fresh Meters and on one cumulative Meter, and compare every charge point, limit kind, amount, order, the denied charge and the incomplete record.
5. For AC-7 and AC-9, code review inspects the proof-path source for `num_bigint`, `Integer`, `Rational`, `Vec`, `cfg(kani)`, iterator adapters, loops (each loop has a stated constant bound and no operand-dependent `break`, `return` or `?`) and calls from a native body to an owner function, and inspects each harness for the functions its expected side shares with the native body (none may be shared) and for its docstring of covered outputs. There is no automated step: a source-text scan is not a test.
6. For AC-8, run a counting global allocator around the native operations over the boundary set, asserting zero allocations, and call each operation and comparison from a host test.
7. For AC-10, run each mutant through the differential and assert it fails at the step the table names.
8. For AC-11, AC-12 and AC-13, run each Kani harness of each operation, comparison and width under the authorized runner with a 10-minute wall-clock limit and record verdict, time and peak memory with its provenance. An arithmetic combination has three harnesses that together cover the six rounding spellings (`NearestEven`; `Exact`; the four directed and nearest-away spellings drawn symbolically), and each harness is its own row; a combination is VERIFIED only when all three are. The Binary64 add and multiply harnesses are the probe: they are the AC-11 harnesses, not separate ones, and each combination is judged alone. For a non-VERIFIED outcome, record the row in the evidence record below and in the Kani tractability table that IR-1526's seam loads. Confirm each VERIFIED harness's cover is reached and its harness mutant fails, and compare each peak with 16 GiB.
9. For AC-14, code review searches the kernel source for a refusal variant, refusal path or `Meter` behaviour defined for the table-row state; none may exist, and the seam referenced is IR-1526's. There is no automated step.
10. For AC-15, inspect the Status for a per-operation, per-width verdict or table row, including SquareRoot and FusedMultiplyAdd.

A Kani harness is a property harness (FR-405 Behavior 19, second sentence): per operation it draws the operands (two, one for Negate and SquareRoot, three for FusedMultiplyAdd) as fully symbolic bit patterns of the width, runs the native operation with a meter that never denies, and asserts the outputs listed in the harness docstring of `src/native_float/kani_proofs.rs` against an expected side that shares no function with the native body: result bits under `NearestEven` and a completed `Exact`, the NaN result (sign, quietness, payload) and the `invalid` flag of NaN-producing operations under every spelling, that only `Exact` refuses, provenance and the result-unit charge. The outputs it does not cover (result bits under the directed and nearest-away spellings, the `overflow`, `underflow`, `inexact` and `divide_by_zero` flags, the `Exact` refusal's would-be flags, charge amounts and meter limits) are left to the host differential. For a comparison it asserts the boolean and the result-unit charge.

### Mutants

| Mutant (site) | Distinguishing input | Fails at |
| --- | --- | --- |
| The tie rule of nearest-even is nearest-away | a tie operand pair whose even neighbour is the lower value | step 3 |
| Nearest-away rounds ties to even | a tie whose larger-magnitude neighbour is odd | step 3 |
| Toward-zero increments the magnitude | an inexact positive result and an inexact negative result under toward-zero | step 3 |
| Toward-positive and toward-negative are swapped | an inexact positive result and an inexact negative result | step 3 |
| The sticky bit is dropped in alignment | an add of one and a tiny operand, one unit off a tie | step 3 |
| The subnormal boundary is off by one | the largest subnormal plus the smallest subnormal | step 3 |
| The overflow result ignores the mode | the largest finite plus itself under toward-zero, toward-positive, toward-negative and each nearest mode, each in both signs (infinity or largest finite) | step 3 |
| The `overflow` flag is not raised, or raised on the largest finite | the largest finite plus itself, and the largest finite plus zero | step 3 |
| An exact zero sum takes the wrong sign | positive one plus negative one in toward-negative, and in the other modes | step 3 |
| The NaN payload is cleared or not quieted | a signalling NaN and a payload NaN as operands, for each operation | step 3 |
| `invalid` is not raised for infinity minus infinity | infinity minus infinity | step 3 |
| `divide_by_zero` is raised for zero over zero | zero over zero, and one over zero | step 3 |
| `underflow` is raised on an exact subnormal result | an exact subnormal sum | step 3 |
| `inexact` is not raised when only the sticky bit is set | a divide whose remainder is nonzero | step 3 |
| `Exact` returns the rounded value | one divided by three under `Exact` | step 3 |
| `Exact` refusal carries bits | the same input | step 3 |
| Square root of a negative nonzero operand does not raise `invalid`, or negative zero loses its sign | the square root of minus one, and of minus zero | step 3 |
| Fused multiply-add rounds the product before the addition | operands whose product is inexact and cancels in the sum | step 3 |
| NumericEqual treats zeros of opposite sign as unequal, or a NaN as equal | minus zero against plus zero, and a NaN against itself | step 3 |
| TotalOrder ignores the sign or the payload of a NaN | a negative NaN against a positive NaN, and two NaN payloads | step 3 |
| BitIdentical compares numeric value | two NaNs with different payloads, and the two zeros | step 3 |
| Negate raises a flag or quiets a NaN | the negation of a signalling NaN | step 3 |
| The provenance of a result is not the owner's | any completed case, the provenance compared | step 3 |
| `ieee.exact-intermediate` is not charged on a finite plan, or is charged on a classification-decided input | a finite add at its exact limit, and Add with a NaN operand, Add of infinity and minus infinity, and Divide by zero, each with a recording `Meter` expecting no `ieee.exact-intermediate` charge | step 4 |
| `ieee.round` is not charged on a finite plan, or is charged on a classification-decided input | the same inputs, expecting `ieee.round` only on the finite plan | step 4 |
| The operand-count amount of the operand charge is wrong for SquareRoot (1) or FusedMultiplyAdd (3) | SquareRoot of four and FusedMultiplyAdd of one, one and one, expecting operand amounts for 1 and 3 operands (CG FR-053-AC-10 b, c) | step 4 |
| The charge of the operand points is dropped, reordered or charged twice | any completed case at its exact limit | step 4 |
| The result charge is dropped or charged at the wrong amount | any completed case at its exact limit and one less | step 4 |
| The first charge is made after the width check, or the denied charge is followed by a later charge | a cross-width add, and a completed case at one less than its first charge | step 4 |
| An ill-typed call charges the Meter | a cross-width add | step 2 |

## Evidence record

The record of every executed harness. The proofs-gen tractability table is the source IR-1526 loads; this section is the copy that can be inspected from this repository, and no criterion depends on the table's path. Each row gives operation, width, rounding spelling group, outcome (VERIFIED, or timeout, memory exhaustion or unreliable; a FAILED verdict is a defect, not a row), time, peak memory with provenance and harness id. A VERIFIED time is Kani's reported verification time followed by the wall time of the whole `cargo kani` invocation; a timeout row's time is the limit it was killed at.

| Operation | Width | Spelling | Outcome | Time | Peak memory | Harness id |
| --- | --- | --- | --- | --- | --- | --- |
| Negate | Binary32 | n/a | VERIFIED | 1.6 s (wall 5.1 s) | 632508416 bytes (0.59 GiB) | native_float_kani_proofs::float32_negate |
| Add | Binary32 | NearestEven | VERIFIED | 48.3 s (wall 52.0 s) | 1192095744 bytes (1.11 GiB) | native_float_kani_proofs::float32_add_nearest_even |
| Add | Binary32 | Exact | VERIFIED | 35.3 s (wall 40.6 s) | 791445504 bytes (0.74 GiB) | native_float_kani_proofs::float32_add_exact |
| Add | Binary32 | directed and nearest-away | VERIFIED | 13.3 s (wall 18.5 s) | 793542656 bytes (0.74 GiB) | native_float_kani_proofs::float32_add_directed |
| Subtract | Binary32 | NearestEven | VERIFIED | 57.5 s (wall 61.3 s) | 1013403648 bytes (0.94 GiB) | native_float_kani_proofs::float32_subtract_nearest_even |
| Subtract | Binary32 | Exact | VERIFIED | 31.6 s (wall 35.2 s) | 650805248 bytes (0.61 GiB) | native_float_kani_proofs::float32_subtract_exact |
| Subtract | Binary32 | directed and nearest-away | VERIFIED | 12.9 s (wall 16.2 s) | 1043410944 bytes (0.97 GiB) | native_float_kani_proofs::float32_subtract_directed |
| Multiply | Binary32 | NearestEven | VERIFIED | 68.0 s (wall 71.4 s) | 893853696 bytes (0.83 GiB) | native_float_kani_proofs::float32_multiply_nearest_even |
| Multiply | Binary32 | Exact | VERIFIED | 19.0 s (wall 22.3 s) | 844947456 bytes (0.79 GiB) | native_float_kani_proofs::float32_multiply_exact |
| Multiply | Binary32 | directed and nearest-away | VERIFIED | 12.0 s (wall 15.4 s) | 615817216 bytes (0.57 GiB) | native_float_kani_proofs::float32_multiply_directed |
| Divide | Binary32 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 671797248 bytes (0.63 GiB) | native_float_kani_proofs::float32_divide_nearest_even |
| Divide | Binary32 | Exact | timeout | 1800 s (limit; killed, no verdict) | 657661952 bytes (0.61 GiB) | native_float_kani_proofs::float32_divide_exact |
| Divide | Binary32 | directed and nearest-away | VERIFIED | 186.4 s (wall 189.8 s) | 712122368 bytes (0.66 GiB) | native_float_kani_proofs::float32_divide_directed |
| SquareRoot | Binary32 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 773021696 bytes (0.72 GiB) | native_float_kani_proofs::float32_square_root_nearest_even |
| SquareRoot | Binary32 | Exact | VERIFIED | 27.7 s (wall 31.1 s) | 818917376 bytes (0.76 GiB) | native_float_kani_proofs::float32_square_root_exact |
| SquareRoot | Binary32 | directed and nearest-away | VERIFIED | 15.8 s (wall 20.0 s) | 825073664 bytes (0.77 GiB) | native_float_kani_proofs::float32_square_root_directed |
| FusedMultiplyAdd | Binary32 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 656998400 bytes (0.61 GiB) | native_float_kani_proofs::float32_fused_multiply_add_nearest_even |
| FusedMultiplyAdd | Binary32 | Exact | VERIFIED | 722.9 s (wall 724.7 s) | 1098571776 bytes (1.02 GiB) | native_float_kani_proofs::float32_fused_multiply_add_exact |
| FusedMultiplyAdd | Binary32 | directed and nearest-away | VERIFIED | 17.2 s (wall 20.7 s) | 795443200 bytes (0.74 GiB) | native_float_kani_proofs::float32_fused_multiply_add_directed |
| NumericEqual | Binary32 | n/a | VERIFIED | 1.7 s (wall 5.0 s) | 607162368 bytes (0.57 GiB) | native_float_kani_proofs::float32_numeric_equal |
| TotalOrder | Binary32 | n/a | VERIFIED | 1.6 s (wall 4.8 s) | 601972736 bytes (0.56 GiB) | native_float_kani_proofs::float32_total_order |
| BitIdentical | Binary32 | n/a | VERIFIED | 1.5 s (wall 4.7 s) | 613216256 bytes (0.57 GiB) | native_float_kani_proofs::float32_bit_identical |
| Negate | Binary64 | n/a | VERIFIED | 1.7 s (wall 5.1 s) | 630165504 bytes (0.59 GiB) | native_float_kani_proofs::float64_negate |
| Add | Binary64 | NearestEven | VERIFIED | 292.6 s (wall 295.9 s) | 920829952 bytes (0.86 GiB) | native_float_kani_proofs::float64_add_nearest_even |
| Add | Binary64 | Exact | VERIFIED | 193.3 s (wall 196.5 s) | 990089216 bytes (0.92 GiB) | native_float_kani_proofs::float64_add_exact |
| Add | Binary64 | directed and nearest-away | VERIFIED | 12.8 s (wall 16.0 s) | 658898944 bytes (0.61 GiB) | native_float_kani_proofs::float64_add_directed |
| Subtract | Binary64 | NearestEven | VERIFIED | 294.8 s (wall 298.2 s) | 868843520 bytes (0.81 GiB) | native_float_kani_proofs::float64_subtract_nearest_even |
| Subtract | Binary64 | Exact | VERIFIED | 211.9 s (wall 215.1 s) | 674004992 bytes (0.63 GiB) | native_float_kani_proofs::float64_subtract_exact |
| Subtract | Binary64 | directed and nearest-away | VERIFIED | 13.0 s (wall 16.3 s) | 861683712 bytes (0.80 GiB) | native_float_kani_proofs::float64_subtract_directed |
| Multiply | Binary64 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 605556736 bytes (0.56 GiB) | native_float_kani_proofs::float64_multiply_nearest_even |
| Multiply | Binary64 | Exact | VERIFIED | 106.2 s (wall 109.6 s) | 609837056 bytes (0.57 GiB) | native_float_kani_proofs::float64_multiply_exact |
| Multiply | Binary64 | directed and nearest-away | VERIFIED | 11.5 s (wall 14.8 s) | 606568448 bytes (0.56 GiB) | native_float_kani_proofs::float64_multiply_directed |
| Divide | Binary64 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 714432512 bytes (0.67 GiB) | native_float_kani_proofs::float64_divide_nearest_even |
| Divide | Binary64 | Exact | timeout | 1800 s (limit; killed, no verdict) | 757579776 bytes (0.71 GiB) | native_float_kani_proofs::float64_divide_exact |
| Divide | Binary64 | directed and nearest-away | timeout | 1800 s (limit; killed, no verdict) | 709488640 bytes (0.66 GiB) | native_float_kani_proofs::float64_divide_directed |
| SquareRoot | Binary64 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 953933824 bytes (0.89 GiB) | native_float_kani_proofs::float64_square_root_nearest_even |
| SquareRoot | Binary64 | Exact | timeout | 1800 s (limit; killed, no verdict) | 925200384 bytes (0.86 GiB) | native_float_kani_proofs::float64_square_root_exact |
| SquareRoot | Binary64 | directed and nearest-away | VERIFIED | 17.6 s (wall 21.1 s) | 986918912 bytes (0.92 GiB) | native_float_kani_proofs::float64_square_root_directed |
| FusedMultiplyAdd | Binary64 | NearestEven | timeout | 600 s (limit; killed, no verdict) | 669093888 bytes (0.62 GiB) | native_float_kani_proofs::float64_fused_multiply_add_nearest_even |
| FusedMultiplyAdd | Binary64 | Exact | timeout | 1800 s (limit; killed, no verdict) | 650145792 bytes (0.61 GiB) | native_float_kani_proofs::float64_fused_multiply_add_exact |
| FusedMultiplyAdd | Binary64 | directed and nearest-away | VERIFIED | 18.4 s (wall 21.7 s) | 706449408 bytes (0.66 GiB) | native_float_kani_proofs::float64_fused_multiply_add_directed |
| NumericEqual | Binary64 | n/a | VERIFIED | 1.8 s (wall 5.0 s) | 602075136 bytes (0.56 GiB) | native_float_kani_proofs::float64_numeric_equal |
| TotalOrder | Binary64 | n/a | VERIFIED | 1.7 s (wall 4.8 s) | 601190400 bytes (0.56 GiB) | native_float_kani_proofs::float64_total_order |
| BitIdentical | Binary64 | n/a | VERIFIED | 1.5 s (wall 4.8 s) | 603111424 bytes (0.56 GiB) | native_float_kani_proofs::float64_bit_identical |

Runner and provenance. Every row was executed under IR-1446 and IR-258's shared PR on the harnesses of `src/native_float/kani_proofs.rs` as committed in that PR. Each row is one run of `cargo kani --harness native_float_kani_proofs::<id> --exact --jobs 1 --solver cadical`, one job at a time under `flock /tmp/cargo-slot.lock` and `systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0`, wrapped in `timeout 600`. Peak memory is the scope's cgroup `memory.peak`, sampled while the run was alive (every 0.3 s; the last sample is the peak recorded); the scope held the whole `cargo kani` invocation, so it includes the build and is an upper bound on the solver's peak. The wall time includes about 3 s of compilation and translation before the solver starts.

What changed since the first record. The first record's harnesses let the `Result<Outcome<..>, ..>` of the operation drop by value; CBMC then unwound the drop glue of the refusal's heap arms (about 3,000 loop iterations) and 26 of the 31 timeout rows never reached the solver. The harnesses now hold the result in `ManuallyDrop`, match by reference, and `check` takes `&Result<..>`; the kernel is unchanged. All 26 rows were rerun: 19 now return VERIFIED (3 of 3 covers, 5 of 5 for a fused harness) and 7 are still timeout rows, which now do reach the solver (every CNF was built, 310 thousand to 511 thousand variables) and hang in it. The five NearestEven rows that already reached the solver (Divide and SquareRoot at both widths, Multiply at Binary64) were rerun and are still timeout rows with 248 thousand to 500 thousand variables: each waits on one query, a rounding bracket that needs a multiplier argument, which is a deductive problem and waits on a deductive prover (the Verus lane). The 12 non-VERIFIED rows are all of this kind; their peaks are all below 1 GiB, so none is a memory row and none is near the 16 GiB ceiling. The rows that were VERIFIED in the first record and whose harness text changed (NearestEven Add and Subtract at both widths, NearestEven Multiply at Binary32) were rerun and are VERIFIED again, as are the comparison and Negate rows.

Limits. Every row was first run under a 600 s limit. The five Exact and directed rows that had not returned (Divide Binary32 Exact, Divide Binary64 Exact and directed, SquareRoot Binary64 Exact, FusedMultiplyAdd Binary64 Exact) and FusedMultiplyAdd Binary32 Exact were run again under a 1800 s cap: FusedMultiplyAdd Binary32 Exact returned VERIFIED at 722.9 s, and the other five are timeout rows at 1800 s. The seven NearestEven timeout rows were measured at the 600 s limit and were not extended: they are the solver hangs above and stay measured not-tractable rows.

Fused multiply-add. The expected side of the host `mul_add` is CBMC's `fma` model, which has two faults of its own (the table "Tool faults" below, each reproduced with no native body). The fused harnesses therefore decide the two faulty regions by IEEE 754 rule, in `fused_multiply_add_expected` (an infinite addend gives the addend; a zero product gives the addend, or the signed zero the sum rule picks when the addend is zero too), and use the model as the oracle everywhere else. No kernel failure is recorded for any fused row: the kernel agrees with the rule in both regions on every path the solver explored. With the faults removed, the directed-spelling fused harness is VERIFIED at both widths and the `Exact` fused harness is VERIFIED at Binary32; the other three fused harnesses are timeout rows.

Expected side of the harnesses. The harnesses compare the native operation with the host language's own `f32`/`f64` operation for Add, Subtract, Multiply and FusedMultiplyAdd (`+`, `-`, `*`, `mul_add`), computed only for operands whose result is not a NaN (a predicate over the bits decides the NaN cases first, so no NaN-producing host operation runs and Kani's default NaN check cannot fail). For SquareRoot and Divide the expected side is an exact-integer bracket of the correctly rounded result: Kani's model of the host `/` is not bit-exact (under symbolic operands it reports `0x800000A9 / 0x9C873333` as other than `0x1BA00000`, which the hardware, the native body and the same expression over concrete operands all give; the same defect was found as a FAILED verdict of the first Binary32 divide harness and measured on a scratch harness before the bracket replaced it), and its model of `sqrt` is not exact either. The comparison harnesses assert the boolean and the result-unit charge against the host `==`, bit equality and a signed-integer fold of the sign-magnitude order. Negate asserts bitwise sign negation with no flag. None of these shares a function with the native body, not even the bit-pattern decode.

### Vacuity and harness mutants

| Harness id | Covers reached | Mutant | Verdict |
| --- | --- | --- | --- |
| `native_float_kani_proofs::float32_negate` | 1 of 1 | negate-quiets | FAILED |
| `native_float_kani_proofs::float32_add_nearest_even` | 3 of 3 | tie-even-is-away | FAILED |
| `native_float_kani_proofs::float32_add_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_add_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_subtract_nearest_even` | 3 of 3 | tie-even-is-away | FAILED |
| `native_float_kani_proofs::float32_subtract_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_subtract_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_multiply_nearest_even` | 3 of 3 | tie-even-is-away | FAILED |
| `native_float_kani_proofs::float32_multiply_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_multiply_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_divide_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_square_root_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_square_root_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_fused_multiply_add_exact` | 5 of 5 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_fused_multiply_add_directed` | 5 of 5 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float32_numeric_equal` | 2 of 2 | numeric-equal-zeros | FAILED |
| `native_float_kani_proofs::float32_total_order` | 2 of 2 | total-order-sign | FAILED |
| `native_float_kani_proofs::float32_bit_identical` | 2 of 2 | bit-identical-numeric | FAILED |
| `native_float_kani_proofs::float64_negate` | 1 of 1 | negate-quiets | FAILED |
| `native_float_kani_proofs::float64_add_nearest_even` | 3 of 3 | tie-even-is-away | FAILED |
| `native_float_kani_proofs::float64_add_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_add_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_subtract_nearest_even` | 3 of 3 | tie-even-is-away | FAILED |
| `native_float_kani_proofs::float64_subtract_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_subtract_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_multiply_exact` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_multiply_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_square_root_directed` | 3 of 3 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_fused_multiply_add_directed` | 5 of 5 | nan-payload-cleared | FAILED |
| `native_float_kani_proofs::float64_numeric_equal` | 2 of 2 | numeric-equal-zeros | FAILED |
| `native_float_kani_proofs::float64_total_order` | 2 of 2 | total-order-sign | FAILED |
| `native_float_kani_proofs::float64_bit_identical` | 2 of 2 | bit-identical-numeric | FAILED |

Vacuity and harness mutants (FR-405-AC-12). Each of the 13 VERIFIED harnesses has the cover properties listed above (every `kani::cover!` of the harness is reached: 1 for Negate, 2 for each comparison, 3 for each arithmetic harness) and fails a buildable exact-text mutant of the native body, run by `scripts/ir1446_native_float_kani_mutants.py <harness id>` under the same runner and reporting `VERIFICATION:- FAILED`. The mutants are the host mutants of the table of the same names (`negate-quiets`, `numeric-equal-zeros`, `total-order-sign`, `bit-identical-numeric`, `tie-even-is-away`), so each harness mutant is also an AC-10 mutant. All 13 failed their harness. A harness without a VERIFIED verdict has no vacuity record and claims no proof.

Host mutants (FR-405-AC-10). `scripts/ir1446_native_float_mutants.py` builds 49 mutants in scratch copies (one per row of the table above that has a buildable form, one per charge point of `ieee.rs` including `ieee.exact-intermediate` and `ieee.round` each made on a classification-decided input) and requires the named step's test to fail; all 49 were killed at the step they name and none survived. The table rows that name a source inspection (a loop with an operand-dependent exit, a `cfg(kani)` body, a native body calling the owner, a harness checking the native body against itself, a refusal variant for the table-row state) have no mutant: FR-405-AC-7, AC-9 and AC-14 are inspections by code review and have no automated step.

### Tool faults

Two faults of CBMC's `fma` model, each reproduced by a harness that runs the model alone with no native body (`tool_fault_fma_model_infinite_addend_f32` and `tool_fault_fma_model_zero_product_f32` in `src/native_float/kani_proofs.rs`). They are expected to FAIL and are not defects of the kernel; they are listed so that the model, and not the native fused multiply-add, is named as the source of the fused failure. Time is Kani's verification time; peak memory is the scope's `memory.peak` read inside the scope, which includes the build from cache.

| Harness id | Verdict | Claim and failing check | Time and peak memory |
| --- | --- | --- | --- |
| `native_float_kani_proofs::tool_fault_fma_model_infinite_addend_f32` | FAILED | For finite `x` and `y`, `x.mul_add(y, +inf)` is `+inf`; the model's check "floating-point exception" fails (it raises the exception for any infinite addend, including valid sums) | 0.02 s, 38359040 bytes (0.04 GiB) |
| `native_float_kani_proofs::tool_fault_fma_model_zero_product_f32` | FAILED | For finite `b` and finite nonzero `c`, `0.0.mul_add(b, c)` is `c`; the model returns another finite value, so "0 * b + c is c" fails | 0.02 s, 39030784 bytes (0.04 GiB) |

## Expected Results

Every boundary case yields the owner's bits, flags, provenance and charges, with independent literals for the cases named. Every mutant fails as named. Each combination of operation and width ends as an executed VERIFIED verdict or a measured table row, judged alone, with its peak recorded and at most 16 GiB; a combination with neither is inconclusive and no proof is claimed.

## Status

Run under IR-1446 (reopened) with IR-258 batched into the same PR. The host tests (`tests/ir1446_native_float.rs`, steps 1 to 4, 6, 7 and the evidence checks of step 8) pass. All 44 Kani harnesses of step 8 were rerun on the harness shape described in the runner paragraph, to a terminal state or to the limit it names; none returned a FAILED verdict. 32 are VERIFIED, each with its covers reached and its harness mutant failing: Negate, NumericEqual, TotalOrder and BitIdentical at both widths; Add and Subtract at both widths and Multiply at Binary32 in all three spelling groups; Multiply at Binary64 in the `Exact` and directed groups; Divide at Binary32 in the directed group; SquareRoot at Binary32 in the `Exact` and directed groups and at Binary64 in the directed group; FusedMultiplyAdd at Binary32 in the `Exact` and directed groups and at Binary64 in the directed group. The other 12 are timeout rows, each hanging in the solver (the Exact and directed rows among them at the 1800 s cap, the seven NearestEven rows at the 600 s limit). Per combination: Negate, NumericEqual, TotalOrder, BitIdentical, Add and Subtract at both widths and Multiply at Binary32 are VERIFIED; Multiply at Binary64, Divide, SquareRoot and FusedMultiplyAdd at both widths are table rows, because each has a spelling group that is not verified. No proof is claimed for a table-row combination as a whole; the VERIFIED harnesses of such a combination cover only what the harness docstring lists. The table rows are for IR-1526 (the owner of the refusal seam and of the tractability table) to load; this repository does not own that table, and the evidence record above is the copy. The unverified rows wait on a deductive prover (the Verus lane), or, for the fused rows, on a model of `fma` that is correct for an infinite addend and a zero product.
