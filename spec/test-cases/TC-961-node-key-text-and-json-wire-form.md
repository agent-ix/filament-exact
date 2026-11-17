---
id: TC-961
title: "A node key round trips through its JSON and compact text forms and each wire refusal is exact"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-413
    type: verifies
---
# TC-961: A node key round trips through its JSON and compact text forms and each wire refusal is exact

## Description

Verify FR-413-AC-1 through FR-413-AC-10 with the `filament_exact::node_wire` tests that move from `fsl-foundation::node_wire`, built with the `json` feature: the JSON and compact text forms, the round trip at depth, every `NodeWireError` refusal, and the budgeted reader's admission and cancellation.

## Test Procedure

1. Run `the_wire_shape_carries_package_path_and_ordinal_only` (AC-1) and `deep_text_round_trips_and_matches_the_value_spelling` (AC-2, AC-3).
2. Run `a_nested_identity_round_trips`, `two_thousand_deep_text_reads_in_one_pass` (AC-3) and `nesting_far_beyond_any_small_depth_round_trips_without_stack_growth` (AC-4).
3. Run `malformed_shapes_are_refused_by_member` (AC-5) and `an_inadmissible_identity_is_refused_by_the_key` (AC-6).
4. Run `malformed_text_is_refused` (AC-7).
5. Run `text_parser_admits_real_blocks_before_allocating_and_preserves_caller_cancellation`, which covers the escapes (AC-8), the short budget (AC-9) and the cancelled budget (AC-10).

## Expected Results

1. The JSON object has exactly `package`, `path` and `ordinal`; the text equals `serde_json`'s output of it.
2. Every key round trips equal, at 200, 2,000 and 10,000 levels.
3. Each JSON refusal is `Shape`, `Package`, `Path`, `Ordinal` or `Key(..)` as FR-413 Behavior 7 and 9 state, with no key.
4. Each malformed text is an error and the any-order text with whitespace is accepted.
5. The escaped text reads to the same key as the JSON value; a short budget returns `Denied` with the same last reservation as the full read; a cancelled budget returns `Denied` before any reservation.
