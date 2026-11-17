---
id: SR-17940486709564847048044
title: "IR-1544 PR #175 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #175 at e6624e2a (diff origin/main...HEAD, 20 files); IR-1544; src/native_fraction.rs, src/native_fraction/verus_proofs.rs, src/native_rational.rs, src/native_rational/verus_proofs.rs, src/native_rational/kani_proofs.rs, src/numeric.rs, src/accounting.rs, src/accounting/units.rs, src/lib.rs, src/verus_kernel_model.rs, src/verus_std_specs.rs, Cargo.toml, Makefile, scripts/verus.sh, scripts/ir1544_verus_mutants.py, scripts/ir258_u8_tier_mutants.py, tests/ir258_native_rational.rs, docs/verus-proofs.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-408
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-956
    type: reviews
---

## Summary

Ticket: IR-1544. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (20 files, +3472/-15).

Author's claim against the diff. The PR touches more than the four named source files: it also changes `src/lib.rs` (`#![forbid(unsafe_code)]` becomes `cfg_attr(not(verus_keep_ghost), forbid(unsafe_code))` plus `deny` under Verus), `Cargo.toml` (the `check-cfg` list gains `cfg(verus_keep_ghost)`), adds one Kani harness (`native_rational_equal_all_parts_is_structural`), one row to `scripts/ir258_u8_tier_mutants.py`, and bumps a count in `tests/ir258_native_rational.rs` from 9 to 10. None of these changes the compiled behaviour of a normal build. The forbid still applies to every non-Verus build.

Ghost-only check. Every changed line in a production file is a `cfg_attr(verus_keep_ghost, ...)` attribute, a `#[cfg(verus_keep_ghost)]` item or `proof!` statement, or the one alias in `form` (`let refuse = NativeRationalRefusal::Capacity;` became `use NativeRationalRefusal::Capacity as Refuse;`). The alias names the same tuple-variant constructor, so its semantics are identical. `KERNEL_VERSION` is unchanged and `Cargo.lock` is unchanged. No dependency was added: `vstd` is supplied only by the Verus driver.

Escape hatches, listed. No `assume(`, `admit(`, `#[verifier::external_body]` on a function, or axiom appears in either `verus_proofs.rs`. The trusted base is two files: `src/verus_std_specs.rs` has 8 `assume_specification`s (`i64::unsigned_abs`, `i128::unsigned_abs`, `i128::wrapping_neg`, `u128::leading_zeros`, `Ordering::is_lt/is_le/is_gt/is_ge`), and I checked each against the core semantics (the `u128_leading_zeros` recursion gives 128 at 0 and 127 at 1). `src/verus_kernel_model.rs` has 1 `assume_specification` (`Meter::charge_units` appends to an `uninterp` admitted sequence on Ok and leaves it unchanged on Err), plus `external_type_specification` / `external_body` on the opaque types `Meter`, `Incomplete`, `Undefined` and `Refusal`. Being opaque, these types carry no claim. `verus_verify(external)` excludes `from_rational`, `to_rational`, `UnitsCharge::{equality_plan_form, equality_pair, equality_result_retain, work, from_charge}` from verification, and none of them is in a claimed row. Three `#[verifier::opaque]` spec fns (`gcd64`, `gcd8`, `gcd256`) only hide definitions from the solver and are revealed where they are used. None of these assumptions carries a row's proof by itself, and docs/verus-proofs.md lists them all.

Reviewer runs, in a private detached worktree with its own target dirs, under `flock /tmp/cargo-slot.lock`:

- `make verus` with the locally present Verus binary: `verification results:: 208 verified, 0 errors`, 34.3 s wall including the dependency build, 0.55 GiB peak RSS.
- `make verus-mutants`: unmutated copy verified, and the script reports 17 of 17 killed, 198 s wall. 13 were killed by `postcondition not satisfied`. 4 were killed only by `resource limit (rlimit) exceeded` (FND-001).
- Reviewer mutants (my own list, not the author's). Vacuity probes: `assert(false)` inserted in `row_add`, `row_order` and `row_from_i64_parts` each fails, so the requires are not contradictory. Weakened or mutated ensures: add denominator `p*q+1`, order cross products swapped, equality lemma `a*c == q*p`, spec `canonical` with `gcd <= 2`; each fails. Code mutants: `is_canonical_pair` with `denominator < 0`, `bits` with `.max(0)`, ordering operands charged at the arithmetic point; each fails with a real error. `gcd256` final `g * (u | v)` is killed only by rlimit (FND-001).
- Gates: see Verdict.

## Verdict

The proofs are real. They are attached to the production functions by attributes. They verify. The rows are not vacuous, and the stated ensures fail when the code or the ensures are mutated. The diff is ghost-only for every non-Verus build. Two medium findings concern evidence quality and the repo's rules: mutant kills by solver resource exhaustion, and a hard version pin plus tool versions in the evidence record. One low finding is a churn source in `scripts/verus.sh`. Gates at head are recorded in the Linear comment.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 4 of the 17 recorded Verus mutants (coprime-both-even-test-wrong, ladder-64-steps-code-and-spec, gcd64-power-of-two-not-restored, reduce-wide-tier-keeps-denominator-code-and-spec) are "killed" only by `resource limit (rlimit) exceeded`, not by a failed postcondition or assertion. A reviewer mutant (`gcd256` returning `g * (u \| v)`) behaves the same way. `scripts/ir1544_verus_mutants.py` counts any line starting with `error` as a kill. A solver resource-out is the Verus analogue of a Kani TIMED OUT, which this repo records as "claims no proof", so these 4 kills are not demonstrated. TC-956 ("each makes scripts/verus.sh report an error") and FR-408-AC-13 ("every mutant ... fails them") overstate. Require a semantic error (postcondition, assertion or precondition) for a kill, and record rlimit outcomes as unreliable. Alternatively, re-run those mutants with a higher `--rlimit` until they give a real verdict. | scripts/ir1544_verus_mutants.py:107-115, spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:141 |
| FND-002 | medium | Tool pin and tool versions in the evidence. `scripts/verus.sh` hard-rejects every Verus except `PINNED="0.2026.11.02.8bf68ee"`. The Makefile comment, docs/verus-proofs.md, FR-408-AC-13 ("under the pinned Verus release") and TC-956 step 12 instruct that pin. The TC-956 Verus record names `Verus 0.2026.11.02.8bf68ee (release build, Z3 4.16.0, Rust toolchain 1.99.1-...)`, and no Kani record in TC-956 names tool versions. The standing rules are no pins without permission and no tool versions in run-evidence docs. Drop the version gate (or make it a warning), drop the version strings from FR/TC text, and leave the run's identity to the commit. | scripts/verus.sh:13, Makefile:105, docs/verus-proofs.md:14, spec/functional/FR-408-kani-tractable-native-rational-carrier.md:118, spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:29, spec/test-cases/TC-956-kani-tractable-native-rational-carrier.md:123 |
| FND-003 | low | `scripts/verus.sh` hard-codes the dependency list (`allocator_api2 arrayvec num_bigint ... unicode_normalization`) and picks `find ... \| head -n 1` from `target-verus/debug/deps`. Adding or dropping a dependency breaks the lane. A stale second rlib of the same crate (after a version bump) can be picked nondeterministically. Derive the externs from `cargo build --message-format=json` (compiler-artifact filenames) instead. | scripts/verus.sh:40-50 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/native_rational.rs form alias | examined | `use NativeRationalRefusal::Capacity as Refuse;` replacing `let refuse = NativeRationalRefusal::Capacity;`, same constructor |
| src/lib.rs unsafe_code lint | examined | `#![cfg_attr(not(verus_keep_ghost), forbid(unsafe_code))]` / `#![cfg_attr(verus_keep_ghost, deny(unsafe_code))]` |
| src/verus_std_specs.rs | examined | 8 `assume_specification`s of core functions, each checked against core semantics |
| src/verus_kernel_model.rs | examined | `assume_specification[ Meter::charge_units ]`: Ok pushes `charge` to `admitted`, Err leaves it unchanged; opaque external types |
| src/native_rational/verus_proofs.rs row_from_i64_parts | examined | ZeroDenominator for d == 0; NonCanonical when !canonical; Ok with parts (n, d) when canonical |
| src/native_rational/verus_proofs.rs row_add/row_subtract/row_multiply | examined | r is Ok; Completed or Incomplete; Completed => parts * gcd == exact parts, canonical, four requests in order; Incomplete => proper prefix |
| src/native_rational/verus_proofs.rs row_order | examined | r is Ok; Completed => holds == order_holds(op, a*q, c*p), three requests; Incomplete => prefix |
| src/native_rational/verus_proofs.rs row_equal_structural_is_cross_product | examined | i64 carriers: parts equal <==> a*q == c*p |
| src/native_fraction/verus_proofs.rs gcd/canonical/reduce/coprime/gcd64/gcd8/gcd256 specs and lemmas | examined | spec `gcd` is Euclid over nat; `lemma_coprime`, `lemma_is_canonical_pair`, `lemma_reduce`, `lemma_gcd64/8/256` |
| src/native_rational/kani_proofs.rs native_rational_equal_all_parts_is_structural | examined | every i64 part, sufficient meter: equal == (a == c && p == q), 4 admissions, counters 1/5/1, two covers |
| scripts/verus.sh, scripts/ir1544_verus_mutants.py, Makefile verus targets | examined | version gate, extern list, kill rule `line.startswith("error")` |
| tests/ir258_native_rational.rs pairs count | examined | `assert_eq!(pairs, 10);` |
| Cargo.toml check-cfg | examined | `check-cfg = ['cfg(kani)', 'cfg(verus_keep_ghost)']` |

## Dispositions

Round 1, fix head ea7e1614 (reviewed head e6624e2a). origin/main has moved to 234f233 since the review; `git merge-tree` against it is clean. Reviewer re-runs in a fresh private worktree under flock /tmp/cargo-slot.lock: make verus 211 verified, 0 errors (6.0 s wall, 0.55 GiB); make verus-mutants: unmutated copy verified, 18 of 18 killed, every one by `postcondition not satisfied`, 58 s wall. Probe: the PR's mutant script, run on a whitespace-only mutant under `--rlimit 0.05` so that only resource-limit errors appear, reports `UNRELIABLE` and exits 1. Reviewer re-mutation after the opaque/reveal restructuring: the assert(false) vacuity probes in row_add, row_order and row_from_i64_parts each fail; the mutated ensures (add denominator, swapped order cross products, equality lemma, canonical with gcd<=2) and code (denominator<0, bits max(0), steps8_16 cut to three quarters, ordering charge point) each fail a postcondition, so no ensures was weakened. Observation, not a finding: a code-only edit of an unrolled ladder (steps_128 or gcd_steps_256 cut to one half, spec unchanged) still ends only in rlimit. That is not a pass, so soundness holds, and the recorded mutants of those ladders edit code and spec together and fail the theorem. Gates at ea7e1614: cargo fmt --check clean; make lint and clippy --locked --all-targets -D warnings pass; make test 88 result lines, 0 failed; build-no-std, docs and deny (advisories ok, bans ok, licenses ok, sources ok) pass; git status clean. The committed spec/reviews/ir-1544-pr175-*.md copies were byte-identical (cmp) to the reviewer's round-0 files.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | after: verdict = [line for line in out.splitlines() if line.startswith(SEMANTIC)] ... elif not verdict:     # A solver that ran out of resources gave no verdict; like a     # Kani timeout, that claims nothing.     failures += 1     print(f"{ident}: UNRELIABLE (no failed proof obligation, only a resource limit)") |
| FND-002 | fixed | after: if ! command -v "$VERUS" >/dev/null 2>&1; then   echo "verus.sh: no Verus binary at '$VERUS'; set VERUS to one" >&2   exit 2 fi (no PINNED gate; TC-956 Verus record: "Run by `scripts/verus.sh` (`make verus`) on the source of this change.") |
| FND-003 | fixed | after: cargo metadata --locked --format-version 1 >"$metadata" cargo build --locked --lib --quiet --target-dir "$TARGET" --message-format=json /   python3 -c '... if message.get("reason") != "compiler-artifact" or message["package_id"] not in wanted: ...' |
