---
id: SR-17940486709564847048047
title: "filament-exact PR #180 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #180 at d436f9c0, git diff origin/main...d436f9c0 (base 16eba89a); QSL-2173, IR-1544; src/equality.rs, tests/composite_declarations.rs, tests/union.rs, tests/qsl2171_undefined_no_element.rs, tests/qsl2326_live_occurrences_limit.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-375
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-377
    type: reviews
  - target: ix://agent-ix/filament-exact/FR-379
    type: reviews
---

## Summary

Ticket: QSL-2173 (float membership keys); the PR also refs IR-1544. Code review with the rust-review lane folded in, scoped to `git diff origin/main...d436f9c0` (8 files, +57/-21). The PR fixes a red main.

Root cause 1, confirmed by bisect. At `d99c707^` (e182c65) all 21 `equality::` lib tests pass with `--features test-support`. At `d99c707` (#173) three fail: `kernel_result_bearing_evaluation_returns_relation_plan_and_charge_order` (left 16, right 17), `plan_reservation_refusals_are_atomic_before_pair_charges` and `pair_and_result_refusals_stop_at_the_first_denied_charge`. #173's sub-commit "Count each equality pair once" moved `pairs.add(1)` from the top of the pop loop to after the match. That fixed the MapKeys double count, but it also stopped counting the `Absent/Absent`, `Null/Null` and mismatched slot arms, which `continue` before the add. The E21 list is eight two-slot composites whose last tail is `Absent`. Its count is 1 root + 8 head slots + 8 tail slots = 17, and dropping the final `Absent/Absent` gives 16. So this is a production regression.

The fix in `src/equality.rs:239-248` adds one pair in the `Absent/Absent | Null/Null` arm and in the mismatched-slot arm. Arm by arm, it matches the pre-#173 count:

- `Present/Present` slot: counted once by the shared add after the match (as before).
- `Values`: counted once (as before).
- `Absent/Absent`, `Null/Null`, mismatched kinds: counted once, restored.
- `MapKeys`: adds only `key_plan.pairs`, with no extra 1. #173's fix is kept, and `map_equality_matches_entries_by_key_and_checks_map_type_admission` still asserts 5 pairs for a two-entry map.
- Nested: the walk is unchanged.

FR-376-AC-4 says the pair counts "are those of the same plan before this requirement", and FR-375-AC-9/10 and TC-932 say 17. The restored count matches all three.

Root cause 2: #174 (0fe3d91) changed `sort_by_key` to `compare_membership_keys`, so a set of float-bearing members now forms. The float behaviour is spec-backed: QSpec FR-144-AC-6 says a set element type holding a float is admitted and keyed by the FR-148 `totalOrder` key, FR-144-AC-25 sets the `Set<Float64>` visiting order, and FR-149-AC-22/26 admit `Set<F>` and `union U { V(Float64), N }` members and give `set[+0, -0] = set[-0, +0]`. Production should not go back to refusing float sets. The two integration tests are updated in the right direction. `compare_keys` is still asserted `None`, and the composite/integer and mismatched-slot refusals are kept for both `form` and `form_grouped`. The two findings below are about how strong the new float assertions are.

The `cargo fmt` changes to `tests/qsl2171_undefined_no_element.rs` and `tests/qsl2326_live_occurrences_limit.rs` are whitespace only.

Gates and mutants: see Verdict.

## Verdict

PASS with findings. The counting fix is correct and minimal, and #173's MapKeys fix is kept. The float-keying behaviour it adapts to is spec-backed (QSpec FR-144-AC-6/AC-25, FR-149-AC-26). Two test-strength findings (FND-001 medium, FND-002 low) and one stale doc comment (FND-003 low, outside the diff).

Gates at d436f9c0, run in a private detached worktree under `flock /tmp/cargo-slot.lock` (CARGO_TARGET_DIR in the worktree):

- `make fmt-check`, `make lint` (three clippy runs), `make build-no-std` and `make docs`: all pass.
- `cargo test`: 605 passed. `cargo test --features test-support`: 618 passed.
- `cargo deny check`: passes, with license and wildcard warnings only and no advisory failure.
- `git status`: clean. Cargo.lock and deny.toml are unchanged.

Mutants on the counting arms (`--features test-support`):

- Drop the Absent/Null add: KILLED by the three E21 tests.
- Double-count MapKeys: KILLED by the map pair-count test.
- Drop the mismatched-slot add: SURVIVED. See gap-analysis FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `ieee_union_payload_has_no_key_and_sequence_keeps_occurrences` passes two identical union values (`vec![value.clone(), value.clone()]`) to `form_grouped` for a `Set` and asserts `len() == 2`. `form_grouped` assumes its occurrences are already distinct members (`src/collection.rs:682`), so the test records a precondition violation (a Set holding two identical members) as expected behaviour. It also checks no order, although the edited FR-379-AC-7 says the set is ordered by its membership key. Use two distinct IEEE payloads (for example `+0` and `-0`) in reverse key order and assert the order; optionally also assert that `form` merges the identical pair to one member (FR-149-AC-26 `set[qnan] = set[qnan]`). | tests/union.rs:344-351 |
| FND-002 | low | `float_set` in `pairs_without_a_key_stay_unkeyed_and_refuse` asserts only `len() == 2`. The members are supplied already in key order (`+0` bits 0, then `1.0` bits `0x3f80_0000`), so a missing or reversed sort would still pass. FR-377-AC-6 and TC-925 step 6 now claim both members are retained in order. Supply them reversed and assert the element order. | tests/composite_declarations.rs:520-537 |
| FND-003 | low | The doc comment of `float_pair_has_no_key` ('matching its exclusion from `=` in `crate::equality`'s leaf match') is stale since #174: float `=` is now admitted (IEEE `numericEqual`, FR-149-AC-25). Outside the diff, but the same root cause as this PR. | src/key.rs:385-386 |

## Scope

| Unit | Role | Location | Excerpt |
| --- | --- | --- | --- |
| plan_pairs_with_relation slot arms | examined | src/equality.rs:217-260 | Pair::Slots(FieldValue::Absent, FieldValue::Absent, _) \| Pair::Slots(FieldValue::Null, FieldValue::Null, _) => { pairs = pairs.add(&Integer::one()); continue; } |
| MapKeys arm | examined | src/equality.rs:227-235 | pairs = pairs.add(&key_plan.pairs); equal &= key_plan.equal; continue; |
| E21 qualification tests | examined | src/equality.rs:909-1220 | e21_list_shape_operands; kernel_result_bearing_evaluation_returns_relation_plan_and_charge_order |
| map pair count test | examined | src/equality.rs:500-530 | plan_equality(&first, &reordered).unwrap().pair_events() == 5 |
| composite float set test | examined | tests/composite_declarations.rs:496-545 | assert_eq!(float_set(true), 2); assert_eq!(float_set(false), 2); |
| union IEEE payload test | examined | tests/union.rs:330-372 | form_grouped(&set, vec![value.clone(), value.clone()], ..) ... assert_eq!(grouped.elements().len(), 2) |
| fmt-only test edits | examined | tests/qsl2171_undefined_no_element.rs, tests/qsl2326_live_occurrences_limit.rs | rustfmt line wrapping only |
| form_grouped precondition | context_only | src/collection.rs:682-691 | Form a collection from occurrences that are already distinct members |
