# Verus proofs

`make verus` runs the deductive proofs of this crate with [Verus](https://github.com/verus-lang/verus). They cover claims a bit-level solver does not close: gcd algorithms proved equal, lowest terms, exact 128-bit products. The first user is the native Rational carrier over the full `i64` domain (FR-408 Behavior 18a, TC-956). The second is the native integer division laws over all of `i128` (FR-409 Behavior 24, TC-957). The third is the native float arithmetic: rounding, divide, square root, multiply and fused multiply-add for both widths (FR-405 Behavior 23, TC-953). The fourth is the native Decimal constructor and normalized view over the full width (FR-402 Behavior 11, TC-950).

The lane is not part of `make ci`, like the Kani lane.

## Running

```bash
VERUS=/path/to/verus-x86-linux/verus make verus
VERUS=/path/to/verus-x86-linux/verus make verus-mutants
```

`VERUS` names a Verus binary. The same runs are bound to their acceptance criteria by the ignored tests of `tests/verus_gate.rs` (`cargo test --test verus_gate -- --ignored`). `scripts/verus.sh` builds the crate's dependencies with the Rust toolchain that binary names (`rustup` must have it), takes the list of dependencies and their built artifacts from cargo, runs Verus on `src/lib.rs`, and fails unless Verus reports no error. A full run takes a few seconds and well under 1 GiB.

The mutant scripts (`scripts/ir1544_verus_mutants.py`, `ir1556`, `ir1551`, `ir1554`) apply each recorded mutant to a scratch copy of `src/` and require a proof to fail on it. Only a failed proof obligation counts (a postcondition, precondition or assertion that does not hold, or a possible overflow). A run that only exhausts the solver's resource limit gave no verdict; the script reports it as UNRELIABLE and fails.

## How a proof attaches to the code

The proofs are about the production functions, not about a copy of them. Nothing they add is compiled by any other build: every item is under `cfg(verus_keep_ghost)`, which only a Verus run sets.

| What | Where | Form |
| --- | --- | --- |
| A specification for a production function | on the function | `#[cfg_attr(verus_keep_ghost, verus_spec(r => ensures ...))]` |
| A lemma the function body needs (a shift in range, a divisor not zero) | one statement in the body | `#[cfg(verus_keep_ghost)] proof! { lemma(...); }` |
| The mathematics, the specification functions and the theorems | `src/<module>/verus_proofs.rs` | a `verus!` block |
| A specification view of private fields | next to the type | a `verus!` block with a `closed spec fn` |
| A loop invariant | on the loop statement | `#[cfg_attr(verus_keep_ghost, verus_spec(invariant ..., decreases ...))]` |

Most production functions are specified as "equal to the specification function of the same name". Verus checks that equality against the body; the theorems are then stated on the specification functions and on wrappers that call the production entry points (`row_*`).

The specifications of the long unrolled ladders (`steps_128`, `steps8_16`, `gcd_steps_256`) and of the three gcds are opaque: the solver sees their bodies only where a proof reveals them. Without that it unfolds hundreds of steps wherever one is named, and a wrong body times out instead of failing. A proof that starts to hit the resource limit after an unrelated edit usually needs a definition hidden the same way.

## What is assumed, not proved

Two files and one constant hold everything the proofs take on trust.

| File | Assumption |
| --- | --- |
| `src/verus_std_specs.rs` | Specifications of `core` functions that Verus's library does not specify: `i64::unsigned_abs`, `i128::unsigned_abs`, `i128::wrapping_neg`, `i128::wrapping_rem`, `u128::leading_zeros`, and `Ordering::is_lt`, `is_le`, `is_gt`, `is_ge`. |
| `src/verus_kernel_model.rs` | `Meter::charge_units` appends an admitted request to the meter's admitted sequence and leaves the sequence unchanged on a denial. Which requests a given meter admits is not modelled; that is FR-359 and its Kani gate. The derived equalities of `DivisionProfile` and `RoundingMode` compare variants. |
| `POWERS_OF_TEN` in `src/native_decimal.rs` | The table holds `10^0` through `10^38`. It is filled by a loop inside a constant initializer, which Verus cannot check; the host test `the_power_table_holds_every_power_of_ten_inside_i128` checks every entry. |

A loop-invariant attribute needs the compiler feature `proc_macro_hygiene`, which `src/lib.rs` enables under a Verus run only.

`#![forbid(unsafe_code)]` becomes `deny` under a Verus run only, because Verus's `assume_specification` expands with `unsafe`; the two files above are the only ones that allow it.

Verus's own library (`vstd`) and Z3 are trusted as shipped.
