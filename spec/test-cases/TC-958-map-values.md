---
id: TC-958
title: "Map values"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-410
    type: verifies
---
# TC-958: Map values

## Description

Verify [FR-410](../functional/FR-410-map-values.md) through the public `from_admitted_map`, `MapValue`, `ValueType::admits`, `plan_equality`, `compare_keys` and collection `form`. Scope: FR-410-AC-1 through FR-410-AC-9.

## Test Procedure

1. For FR-410-AC-1, build `Map<String, Integer>` from `[("b", 2), ("a", 1)]` and from no entries; read `map_type()` and `entries()`.
2. For FR-410-AC-2, build `Map<String, Integer>` from `[("same", 1), ("other", 2), ("same", 3)]` and from `[("x", 1), ("y", 2), ("y", 3), ("x", 4)]`; build `Map<Float64, Integer>` from keys `-0.0` and `+0.0`, and from two NaN keys of one bit pattern.
3. For FR-410-AC-3, build `Map<String, Integer>` from `[("a", 1), (Integer 7, 2)]`.
4. For FR-410-AC-4, call `admits` on `Map<String, Integer>[0, 1]` with each value FR-410-AC-4 names, and `admits` on a `Sequence`, `Set`, `Bag` and `OrderedSet` type with a map value.
5. For FR-410-AC-5, run `plan_equality` over each pair FR-410-AC-5 names.
6. For FR-410-AC-6, build two 1,024-entry `Map<Integer, Boolean>` entry lists in opposite orders and match them through the map entry matcher with a counting key comparator.
7. For FR-410-AC-7, call `compare_keys` on each pair FR-410-AC-7 names.
8. For FR-410-AC-8, `form` a `Set<Map<String, Integer>>` from the three maps FR-410-AC-8 names and read its elements; build the two `Map<String, Map<String, Integer>>` values and run `plan_equality` and `compare_keys` over them.
9. For FR-410-AC-9, build `{-0.0: 1}` and `{+0.0: 1}` over `Map<Float64, Integer>`; run `plan_equality` and `compare_keys` over them and `form` a `Set<Map<Float64, Integer>>` from both.

## Expected Results

1. A `Value::Map` of the declared type with entries `[("b", 2), ("a", 1)]` in that order; the empty list gives an empty map.
2. `DuplicateMember` at `Element(2)` for both string lists; a two-entry float-keyed map for `-0.0`/`+0.0`; `DuplicateMember` at `Element(1)` for the two NaN keys.
3. `TypeMismatch` at `Element(1)`.
4. `true` only for the one-entry map of the declared type; `false` for every other value and for every collection type.
5. Equal for the order-variant pair; unequal for every other pair; no refusal.
6. Equal, with fewer than 32 × 1,024 key comparisons.
7. `Equal`, `Less`, `Less`, `Less` and `None`, in the order FR-410-AC-7 lists them.
8. Two set members, `{a: 0}` first; the nested maps are equal and have an `Equal` canonical key.
9. Equal under `plan_equality`; `compare_keys` is not `Equal`; the set holds two members.
