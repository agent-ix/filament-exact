---
id: TC-960
title: "LimitKind::LiveOccurrences is named live_occurrences and stays outside LimitKind::ALL"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-412
    type: verifies
---
# TC-960: LimitKind::LiveOccurrences is named live_occurrences and stays outside LimitKind::ALL

## Description

Verify FR-412-AC-1 through FR-412-AC-4: the name, order and cumulativity of `LimitKind::LiveOccurrences`, its absence from `LimitKind::ALL` and from the `from_code` reader, that no charge vector or meter carries it, and that a caller builds the session's `Incomplete` record with it.

## Test Procedure

1. Read `LimitKind::LiveOccurrences.as_str()`, compare it with `ValueOccurrences`, order it against `ResultUnits`, and read `is_cumulative()`.
2. Read `LimitKind::ALL` and `from_code` for `live_occurrences` and for each `ALL` member's spelling.
3. Build a `ChargeVector` with `size(LiveOccurrences, 5)` and with `exact_size(LiveOccurrences, 5)` beside one without, read `amount(LiveOccurrences)`, charge it on a `Meter` and read `consumed(LiveOccurrences)`.
4. From an integration test, construct `Incomplete { limit_kind: LiveOccurrences, limit: 1_048_576, consumed: 1_048_576, next_charge: 2, charge_point: FunctionCall }`, wrap it in `Outcome::Incomplete` and read each field.

## Expected Results

1. `live_occurrences`; unequal to `ValueOccurrences`; ordered after `ResultUnits`; not cumulative.
2. `ALL` is the ten `ScalarLimitsV1` counters in field order without `LiveOccurrences`; `from_code("live_occurrences")` is `None` and every `ALL` spelling reads back to its kind.
3. Both vectors equal the vector built without the entry, `amount` is 0, the charge is granted and `consumed` is 0.
4. The record is constructed, carried, and reads back `LiveOccurrences`, 1,048,576, 1,048,576, 2 and `FunctionCall`.
