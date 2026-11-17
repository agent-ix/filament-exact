---
id: FR-410
title: "Map values"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-386
    type: depends_on
  - target: ix://agent-ix/filament-exact/FR-097
    type: references
  - target: ix://agent-ix/filament-exact/FR-377
    type: references
  - target: ix://agent-ix/filament-exact/FR-378
    type: references
  - target: ix://agent-ix/filament-specification/FR-144
    type: references
  - target: ix://agent-ix/filament-specification/FR-149
    type: references
---
# FR-410: Map values

## Description

The `filament-exact` kernel SHALL provide a map value: the variant `Value::Map` holding a `MapValue`, which carries its declared `MapType` (FR-386) and its entries as `(key, value)` pairs. A map is a finite function from keys of type `K` to values of type `V` that holds one entry for each key it holds (QSpec FR-144 "Map values", lines 267-280). The kernel builds a map from caller-admitted entries and refuses a repeated key, admits a map value at its map type, compares two maps independently of entry order (QSpec FR-149 `Map` row, line 92), and gives every map a canonical key over its entries in ascending key order (QSpec FR-144 canonical-key table, line 164), so that a `Set<Map<K, V>>` and a map whose values are maps are ordinary kernel values (QSpec FR-144 lines 294-295).

## Use case

An evaluator reads a parameter `m: Map<String, Integer>[0, 4]` from its run input. It admits each key against `String` and each value against `Integer` by its own rules, builds the value with `from_admitted_map(map_type, entries)`, and receives a refusal if the input repeats a key. It compares `m` with another map by the kernel's equality plan, whatever order either input listed its entries in, and places maps in a `Set<Map<String, Integer>>` whose members the kernel orders and deduplicates by their canonical keys.

## Inputs

- A `MapType` (FR-386) and a list of `(key, value)` entries whose values the caller has admitted at `V`.

## Outputs

- `Value::Map(Arc<MapValue>)`, or a `ConstructionRefusal` naming the entry at fault.
- `MapValue::map_type()` returns the declared `MapType`; `MapValue::entries()` returns the entries in the order supplied.

## Behavior

1. **Structure.** `Value::Map` is a value variant of its own, distinct from `Value::Collection`. A `MapValue` keeps its entries in the order the caller supplied them; that order is not observable through equality or the canonical key (Behaviors 4 and 5). Its occurrence count is one plus the occurrence counts of every key and every value.
2. **Construction.** `from_admitted_map(map_type, entries)` builds a map from entries the caller has admitted, and takes no meter, as the other caller-admitted constructors do (FR-378). It checks, in this order:
   1. each key against `K` with `ValueType::admits`; the first entry `i` whose key `K` does not admit is refused with `ConstructionRefusal { component: Component::Element(i), cause: ConstructionCause::TypeMismatch }`;
   2. that every pair of keys has a canonical key relation (Behavior 3); otherwise the map is refused with `ConstructionRefusal { component: Component::Element(0), cause: ConstructionCause::TypeMismatch }`, because no single entry is at fault;
   3. that no two keys are equal (Behavior 3); otherwise it is refused with `ConstructionRefusal { component: Component::Element(j), cause: ConstructionCause::DuplicateMember }`, where `j` is the smallest index whose key equals the key of an earlier entry. The refused index does not depend on the order in which the kernel examines the keys.

   It does not check the values against `V` or the entry count against the bound: those are the caller's admission, and `ValueType::admits` reports them (Behavior 6).
3. **Key identity.** Two keys are the same key exactly when their canonical keys under the collection membership relation are equal (FR-097), the relation a `Set<K>` deduplicates by. A map's keys are therefore unique under `K`'s canonical key, as QSpec FR-144 requires of a map ("one for each key it holds"). For a `Float32` or `Float64` key this is the IEEE `totalOrder` key, so `-0` and `+0` are distinct keys and a NaN key is the same key as a NaN of the same bit pattern.
4. **Equality.** The kernel's equality plan (`plan_equality`) over two maps follows the QSpec FR-149 `Map` row: the maps are equal exactly when they have the same `MapType`, an equal entry count, identical key sets under Behavior 3, and recursively equal values at every key. Entry order never matters. Two maps of different `MapType`s, or of different entry counts, are unequal (`false`, not a refusal). The plan matches entries by sorting each side's keys once, so the number of key comparisons grows as `n log n` in the entry count `n`, never as `n²`.
5. **Canonical key.** `compare_keys` over two maps follows the QSpec FR-144 `Map<K, V>` row: it sorts each map's entries into ascending key order (Behavior 3), then compares the two entry lists lexicographically, each entry by its key's key and then its value's key, a proper prefix first. Two maps have equal canonical keys exactly when they are equal under Behavior 4, whatever order their entries were supplied in. A map's canonical key exists whenever every value's canonical key exists, so a map can be a `Set`, `Bag` or `OrderedSet` member and a map's value can be a map. A map against a value of another variant has no canonical key (`None`), as for every other variant pair.
6. **Admission.** `ValueType::Map(declared).admits(value)` is `true` exactly when `value` is a `Value::Map` whose `MapType` equals `declared`, whose entry count is within the declared bound (always, when the bound is absent), and whose every key `K` admits and every value `V` admits. Every other value at a `Map` type, and a map value at every other type, is not admitted.
7. **Run form.** The kernel has no map wire encoding. FSL owns the map run form `{"map": [{key, value}]}`, its key-byte ordering and its round trip.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-410-AC-1 | `from_admitted_map` over `Map<String, Integer>` with entries `[("b", 2), ("a", 1)]` returns a `Value::Map` whose `map_type()` equals the declared type and whose `entries()` are `[("b", 2), ("a", 1)]` in that order. The empty entry list builds an empty map. | Test (TC-958) |
| FR-410-AC-2 | `from_admitted_map` over `Map<String, Integer>` with entries `[("same", 1), ("other", 2), ("same", 3)]` returns `ConstructionRefusal { component: Element(2), cause: DuplicateMember }`; with `[("x", 1), ("y", 2), ("y", 3), ("x", 4)]` it returns `Element(2)`; and over `Map<Float64, Integer>` the keys `-0.0` and `+0.0` build a two-entry map, while two NaN keys of one bit pattern return `DuplicateMember` at `Element(1)`. | Test (TC-958) |
| FR-410-AC-3 | `from_admitted_map` over `Map<String, Integer>` with entries `[("a", 1), (Integer 7, 2)]` returns `ConstructionRefusal { component: Element(1), cause: TypeMismatch }`. | Test (TC-958) |
| FR-410-AC-4 | `ValueType::admits` on `Map<String, Integer>[0, 1]` returns `true` for a one-entry map of that type, and `false` for: a two-entry map of that type (bound), a map of `Map<String, Integer>` with no bound and of `Map<String, Boolean>[0, 1]` (type), a map of that type built with an entry whose value is `Boolean` (value), and a `Sequence<Integer>` value. `ValueType::Collection` of every kind returns `false` for a map value. | Test (TC-958) |
| FR-410-AC-5 | Under `plan_equality`, two `Map<String, Integer>` maps built from `[("b", 2), ("a", 1)]` and `[("a", 1), ("b", 2)]` are equal; one built from `[("a", 1), ("b", 3)]` is unequal to both; a map with keys `{a, c}` is unequal to one with keys `{a, b}` and equal values; maps of different entry counts are unequal; and two maps with equal entries and different `MapType`s (bounded against unbounded) are unequal, with no refusal in any case. | Test (TC-958) |
| FR-410-AC-6 | Equality of two 1,024-entry `Map<Integer, Boolean>` maps whose entries are supplied in opposite orders makes fewer than 32 × 1,024 key comparisons and finds them equal. | Test (TC-958) |
| FR-410-AC-7 | `compare_keys` returns `Equal` for the two order-variant maps of FR-410-AC-5, `Less` for `{a: 1, b: 2}` against `{a: 1, b: 3}`, `Less` for `{a: 9}` against `{b: 0}` (key before value), `Less` for `{a: 1}` against `{a: 1, b: 0}` (proper prefix first), and `None` for a map against a `Sequence` value. | Test (TC-958) |
| FR-410-AC-8 | A `Set<Map<String, Integer>>` formed with `form` from the maps `[("b", 2), ("a", 1)]`, `[("a", 1), ("b", 2)]` and `[("a", 0)]` holds two members, `{a: 0}` before `{a: 1, b: 2}`. A `Map<String, Map<String, Integer>>` built from order-variant inner maps is equal under `plan_equality` and has an `Equal` canonical key against the same map built with its inner entries in the other order. | Test (TC-958) |

## Status

Specified, not yet implemented on main. The map run form, its key-byte ordering and its round trip are FSL's, not this kernel's.

## Dependencies

- The map type, its key type and its bound: [FR-386](./FR-386-map-value-type-with-a-key-type.md). Collection kinds, bounds and the membership relation: [FR-097](./FR-097-unbounded-collection-in-the-kernel.md). Composite keys across declarations: [FR-377](./FR-377-composite-canonical-key-across-declarations.md). Caller-admitted constructors: [FR-378](./FR-378-caller-admitted-values-in-collections-options-and-composites.md).
- QSpec `ix://agent-ix/filament-specification/FR-144` "Map values" (lines 267-280) defines the map as a finite function with one entry per key, its canonical-key row (line 164) orders maps, and `Set<Map<K, V>>` is keyed by that row (lines 294-295). QSpec `ix://agent-ix/filament-specification/FR-149` (line 92) defines map equality.
- The falsifying controls are [TC-958](../test-cases/TC-958-map-values.md).
