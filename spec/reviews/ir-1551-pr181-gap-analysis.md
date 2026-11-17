---
id: SR-17940486709564847048051
title: "IR-1551 PR #181 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #181 at 182ab1ce; FR-405 Behavior 23, FR-405-AC-15, FR-405-AC-17, FR-405-AC-18; tests/verus_gate.rs, tests/ir1446_native_float.rs, scripts/ir1551_verus_mutants.py, src/native_float.rs, src/native_float/verus_proofs.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-405
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-953
    type: reviews
---

## Summary

Ticket: IR-1551. Planless gap analysis. Plan completion: not assessed.

The matrix comes from `quoin matrix --repo . --json` (quoin 0.29.1):

- FR-405-AC-17 binds `verus_gate::verus_proofs_verify` and FR-405-AC-18 binds `verus_gate::verus_mutants_are_killed`. Both are `tagged-by-ignored-test`, the same state as the sibling Verus criteria FR-408-AC-13/14 and FR-409-AC-11/12.
- AC-1 to AC-16 keep the bindings they have on main.
- FR-405-AC-15 binds `the_status_records_a_verdict_or_table_row_for_every_operation_and_width`, which this PR breaks (code-review FND-001).

Each function Behavior 23 lists has a `verus_spec` attribute and a theorem: decode, the format helpers, justify, plan_multiply, plan_divide, plan_square_root, integer_square_root, product_term, term, normalised, sum, plan_fma, round_at and round. No production code was added that has no owning requirement. There are no stubs. There are no `todo!()` or `unimplemented!()` calls in the diff.

## Verdict

The trace is complete for AC-17 and AC-18. One medium gap: Behavior 23's last sentence asks for a mutant per covered function, and several covered functions have none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Behavior 23 ends with "Each proof SHALL fail on a mutant of the function it covers", and it lists `term`, `normalised` and the format helpers (`fraction`, `bias`, `quantum_min`, `emin`, `sign_mask`, `exponent_ones`, `exponent_mask`, `fraction_mask`, `quiet_bit`, `sign`, `infinity`, `max_finite`, `canonical_nan`, `zero_bits`) as covered. scripts/ir1551_verus_mutants.py has no mutant of any of them. So AC-18, "every mutant ... makes a proof fail", passes while the obligation of Behavior 23 is not met. Add a mutant per listed function (for example `normalised` one bit short, `max_finite` fraction mask dropped), or narrow the sentence to the functions the script mutates. | spec/functional/FR-405-kani-tractable-native-float-carrier.md:156; scripts/ir1551_verus_mutants.py:22-142 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-405-AC-17 | examined | The Verus proofs of Behavior 23 verify with no error (`make verus`) on the production functions Behavior 23 lists. |
| FR-405-AC-18 | examined | Every mutant of `scripts/ir1551_verus_mutants.py` makes a proof of Behavior 23 fail with a failed proof obligation ... |
| FR-405-AC-15 | examined | No Kani proof is claimed for a combination whose harness has no executed VERIFIED verdict, and the Status records, per operation and width, the verdict or the table row. |
| FR-405 Behavior 23 | examined | ... The proof covers `decode`, the format helpers, `justify`, `plan_multiply`, `plan_divide`, `plan_square_root`, `integer_square_root`, `product_term`, `term`, `normalised`, `sum`, `plan_fma`, `round_at` and `round`. ... Each proof SHALL fail on a mutant of the function it covers. |

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
| FND-001 | fixed | after: Every function Behavior 23 lists is the target of at least one mutant: round_at (three), round (three), plan_divide (two), plan_square_root (one), integer_square_root (two), sum (one), product_term (one), justify (one), term (one), normalised (one), each of the fourteen format helpers (one each), decode (one), plan_multiply (one), plan_fma (one). The reviewer's run killed all 33. |
