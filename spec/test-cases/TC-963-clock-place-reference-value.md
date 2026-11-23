---
id: TC-963
title: "A clock-place reference is a distinct kernel value compared and keyed by place"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-415
    type: verifies
---
# TC-963: A clock-place reference is a distinct kernel value compared and keyed by place

## Description

Verify FR-415-AC-1 through FR-415-AC-6: `ValueType::ClockReference` admits
only clock-place references, the constructor admits exactly the canonical
`place/v2` encoding, a value round trips through its outputs, equality is by
place within one universe and refuses across universes, the key orders by
universe and then by place id octets, and a value charges one occurrence and
its text length.

Fixture: `A` is the literal text of FSL FR-230-AC-43. `A(K=x)` is `A` with
its second component replaced by the lowercase hex of the JCS JSON `x`, and
likewise `A(P=x)`, `A(T=x)` and `A(J=x)` for the first, fourth and fifth
components. `U1` and `U2` are two distinct `UniverseId` fixtures.
`Tq` is `[["field",["source","a","i","ix://i/C"],"ix://i/C/q",["collection","sequence","0",null,["clock"]]],["collection","sequence","0",null,["clock"]]]`.

## Test Procedure

1. For AC-1, build `c = ClockPlaceReference::new(U1, A)` and offer
   `Value::ClockReference(c)` to `ValueType::ClockReference`,
   `ValueType::Reference(EffectiveId::fixture(6))`, `Text`, `String`, `Bytes`
   and `JsonObject`. Offer to `ValueType::ClockReference` an object reference
   `ObjectReference::new(U1, EffectiveId::fixture(6), ObjectId::new(A))` and
   Text, String, Bytes and JsonObject values that hold `A`. Compare two
   `ValueType::ClockReference` values. Build `Option<ClockReference>`,
   `Sequence<ClockReference>[0, 3]` and `Map<String, ClockReference>` values
   holding `c`, and call `MapType::new` with key type `ClockReference`.
2. For AC-2, construct `A` and `A(T=Tq, J=[["slot","1","7"]])` under `U1`.
   Then construct each refused text AC-2 lists: `A` with its first hex
   letter uppercased; `A` with prefix `place/v1/`; `A` without its fifth
   component; `A` followed by `/`; `A` with its third component empty; `A`
   with its third component shortened by one digit; `A` with one digit
   replaced by `g`; `A(K="")`; `A(K=1)`; `A(T=[])`;
   `A(J=[["slot","0","0"]])`; `A(J=[["slot","01","0"]])`; `A` with its fifth
   component `5b205d` (`[ ]`); and `A` with its second component `ff`.
3. For AC-3, for each text step 2 accepts, read the value's text and
   universe, construct a second value from them, plan equality between the
   two values and compare their keys.
4. For AC-4, plan and evaluate equality under `U1` between two values of `A`,
   then between `A` and each of `A(K="c/e")`, `A` with the `field` member
   URI `ix://i/C/y`, and `A(P=["source","a","i","ix://i/other"])`, recording
   the planned pair count and charges. Plan equality between `A` under `U1`
   and `A` under `U2`. Plan equality between `c` and the object reference of
   step 1, a Text holding `A` and a String holding `A`.
5. For AC-5, under `U1` compare keys of `A(K="c")` and `A(K="c/d")`, of
   `A(K="c/d")` and `A(K="c/e")`, of `A(P=["source","a","i","ix://i/b"], K="c/e")`
   and `A(K="c/d")`, of `A` under `U1` and `A` under `U2`, of `A` and `A`,
   and of `c` and the object reference of step 1. Inspect the public API of
   `ClockPlaceReference` and `Value` for an ordering operator. Build a
   `Set<ClockReference>` from the first three values in each of two
   insertion orders.
6. For AC-6, read `occ` and the accounted variable size of `c`, then rerun
   the existing object-reference, scalar, composite, Bytes, JsonObject, UUID
   and Timestamp admission, equality, key and accounting vectors.

## Expected Results

1. Only `ValueType::ClockReference` admits `c`; it rejects the object
   reference and the four text-bearing values; the two types are equal; the
   three wrapper values build and admit; `MapType::new` builds the map type.
2. Both accepted texts construct. Every refused text returns
   `ConstructionRefusal { component: Component::Value, cause:
   PlaceIdNoncanonical }` with no value and no `CheckedInvariant` fault.
3. The text output equals the supplied text byte for byte, the universe
   output equals `U1`, equality is `true` and the key is `Equal`.
4. `A` against `A` is `true` and the three variants are `false`, each with
   the Boolean-leaf pair count and charges. `U1` against `U2` returns
   `Refusal::ForeignReference` whose `code()` is `foreign_reference` and
   `cause()` is `foreign-universe`, with no completed Boolean. The three
   mixed-kind plans return `CheckedInvariant { cause: ValueKindMismatch }`.
5. `A(K="c")` is `Less` than `A(K="c/d")`; `A(K="c/d")` is `Less` than
   `A(K="c/e")`; the value with `P` ending `ix://i/b` is `Less` than
   `A(K="c/d")` although its `K` is greater; `U1` against `U2` follows the
   `UniverseId` order; `A` against `A` is `Equal`; the mixed pair is `None`.
   No ordering operator exists. Both sets hold the three values in key order.
6. `occ` is one and the variable size is 279 bytes; every existing vector
   keeps its result.
