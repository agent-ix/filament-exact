---
id: FR-412
title: "LimitKind::LiveOccurrences names a session's live-occurrence limit outside the scalar counters"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-388
    type: references
  - target: ix://agent-ix/filament-exact/FR-368
    type: references
  - target: ix://agent-ix/filament-eval/FR-011
    type: references
  - target: ix://agent-ix/filament-spec-language/NFR-130
    type: references
---
# FR-412: LimitKind::LiveOccurrences names a session's live-occurrence limit outside the scalar counters

## Description

The `filament-exact` kernel SHALL provide `LimitKind::LiveOccurrences`, the limit kind of an `Incomplete` record that an evaluation session builds when a call would push its live occurrences past the session's live-occurrence limit (filament-eval FR-011). Its normative name is `live_occurrences`. It is not a member of `LimitKind::ALL`, because `ALL` is the ten `ScalarLimitsV1` counters a `Meter` charges, and the live-occurrence limit is a session bound that no meter charges.

## Use case

A session evaluating a deep recursion holds 1,048,576 live occurrences under the default limit, and the next call's frame would add 2. The session builds `Incomplete { limit_kind: LimitKind::LiveOccurrences, limit: 1_048_576, consumed: 1_048_576, next_charge: 2, charge_point: ChargePoint::FunctionCall }` and ends the application incomplete. FSL reads `limit_kind.as_str()` as `live_occurrences` and reports `stage_limit_exceeded` naming `evaluation.live_occurrences`.

## Inputs

- None. `LiveOccurrences` is a unit variant.

## Outputs

- `LimitKind::LiveOccurrences` in an `Incomplete` record's `limit_kind`.

## Behavior

1. **Variant and name.** `LiveOccurrences` SHALL be a unit variant of the public `LimitKind`, declared after `ResultUnits`, so the derived order places it after every scalar counter. `LimitKind::LiveOccurrences.as_str()` SHALL return `live_occurrences`, distinct from `value_occurrences`, which is the scalar counter of one value's occurrences.
2. **Outside `ALL`.** `LimitKind::ALL` SHALL NOT contain `LiveOccurrences`; it stays the ten `ScalarLimitsV1` counters in field order (FR-388). Every per-counter structure the meter sizes from `ALL` keeps ten slots. The reason is that `ALL` is the meter's vocabulary: a `ScalarLimits` has no `live_occurrences` field, a `Meter` charges and denies only `ALL`'s counters, and a session, not a meter, counts live occurrences and applies their limit.
3. **Reader.** `LimitKind::from_code("live_occurrences")` SHALL return `None`, because `from_code` reads exactly the `ScalarLimitsV1` spellings (FR-388 Behavior 5). No other kernel reader accepts the name.
4. **The meter never meets it.** `LimitKind::LiveOccurrences.is_cumulative()` SHALL be `false`: the live count is a high-water size that falls as frames return. `ChargeVector::size` and `ChargeVector::exact_size` with `LiveOccurrences` SHALL leave the vector unchanged; `ChargeVector::amount` and `Meter::consumed` of `LiveOccurrences` therefore return 0, so no meter charge, denial or `Incomplete` the meter builds names it.
5. **Incomplete fields.** An `Incomplete` with `limit_kind: LiveOccurrences` is built by its session, not by a meter, with these meanings: `limit` the session's configured `live_occurrences`; `consumed` the session's live count before the call; `next_charge` the occurrences of the frame the call would push (one plus the `occ` of each value bound at entry), as an `Integer`; and `charge_point` `ChargePoint::FunctionCall`, the point whose granted charge precedes the test. The record carries no other member, as every `Incomplete` does. The `limit_kind` documentation SHALL state that a meter's record names the first unavailable `ScalarLimitsV1` counter and a session's record names `LiveOccurrences`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-412-AC-1 | `LimitKind::LiveOccurrences.as_str()` is `live_occurrences`, the variant is unequal to `ValueOccurrences` and orders after `ResultUnits`, and `is_cumulative()` is `false`. | Test (TC-960) |
| FR-412-AC-2 | `LimitKind::ALL` has exactly the ten `ScalarLimitsV1` counters in field order and does not contain `LiveOccurrences`, and `from_code("live_occurrences")` is `None` while `from_code(kind.as_str())` is `Some(kind)` for every member of `ALL`. | Test (TC-960) |
| FR-412-AC-3 | A `ChargeVector` built with `size(LimitKind::LiveOccurrences, 5)` and with `exact_size(LimitKind::LiveOccurrences, 5)` equals the vector built without it, its `amount(LiveOccurrences)` is 0, and a `Meter` that charged it reports `consumed(LiveOccurrences)` 0 and no denial. | Test (TC-960) |
| FR-412-AC-4 | `Incomplete { limit_kind: LimitKind::LiveOccurrences, limit: 1_048_576, consumed: 1_048_576, next_charge: Integer::from(2), charge_point: ChargePoint::FunctionCall }` is constructible by a caller outside the crate, carried by `Outcome::Incomplete`, and reads back each field unchanged. | Test (TC-960) |

## Status

Specified, not yet implemented on main.

## Dependencies

- [FR-388](./FR-388-read-closed-limit-kind-spellings.md) fixes `ALL` and `from_code` to the `ScalarLimitsV1` vocabulary; [FR-368](./FR-368-meter-charge-and-limit-vocabulary.md) owns the meter's charge and limit vocabulary.
- `ix://agent-ix/filament-eval/FR-011` builds the `LiveOccurrences` record at a call that would exceed the session's limit.
- FSL `ix://agent-ix/filament-spec-language/NFR-130` owns the bound, its count and its default; FSL maps the record to `stage_limit_exceeded` naming `evaluation.live_occurrences`.
- The falsifying controls are [TC-960](../test-cases/TC-960-live-occurrences-limit-kind.md).
