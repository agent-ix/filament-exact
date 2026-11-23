---
id: FR-415
title: "Carry a clock-place reference as a distinct kernel value"
type: FR
relationships:
  - target: ix://agent-ix/filament-spec-language/FR-230
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-106
    type: references
  - target: ix://agent-ix/filament-specification/FR-829
    type: references
  - target: ix://agent-ix/filament-specification/FR-415
    type: references
  - target: ix://agent-ix/filament-specification/FR-322
    type: references
  - target: ix://agent-ix/filament-exact/FR-385
    type: references
  - target: ix://agent-ix/filament-exact/FR-096
    type: references
---
# FR-415: Carry a clock-place reference as a distinct kernel value

## Description

When a caller supplies a `Reference<Clock>` value, the `filament-exact` kernel SHALL
carry it as a clock-place reference of the distinct type
`ValueType::ClockReference`, whose payload is the outer universe and the
canonical `place/v2` place id text of FSL FR-230 "Place identity", and SHALL
admit, compare, key and charge it by that place identity alone. The kernel SHALL
NOT treat a clock-place reference as an object reference, a Text, a String,
Bytes, a JsonObject or a clock value.

## Rationale

`Reference<Clock>` is a data value that names one clock place, compared by
place and never by clock value (FSpec FR-415 "Clock places", FSpec FR-829). A
clock is not an object type, so `ValueType::Reference(EffectiveId)`, which
names an object type and whose value names an object, cannot represent it
faithfully. Without a kernel type, a checked-package consumer cannot project a
clock-reference member to an Exact value type and must refuse it. The checked
package types every `Reference<Clock>`, and `Reference<A>` for an alias `A` of
`Clock`, by one `scalar_type`/`clock_reference` node (FSpec FR-829 "The type
node"), and the place id is alias-erased with the fixed target Clock (FSL
FR-230 "Canonical text grammar"). One unparameterised kernel type therefore
represents every clock-reference type exactly.

## Use case

A timed model's record `Call` has a field `r: Reference<Clock>` that an
operation sets with `self.r = ref(self.x)`. The checked package's projection
gives `r` the type `ValueType::ClockReference`. A snapshot that stores the
place id of `c1.x` in `r` is admitted as one clock-place reference; it equals
another reference to `c1.x`, differs from a reference to `c1.y` or `c2.x`, and
keys a `Set<Reference<Clock>>` in one deterministic order.

## Behavior

1. **One distinct type.** `ValueType::ClockReference` is a public type variant
   with no parameter, and `Value::ClockReference(ClockPlaceReference)` is its
   value variant. `ValueType::admits` accepts a clock-place reference only
   against `ValueType::ClockReference`, and `ValueType::ClockReference`
   rejects every other value kind, including an object `Value::Reference`,
   Text, String, Bytes and JsonObject values that hold the same characters.
   `ValueType::Reference(_)` rejects every clock-place reference. Two
   clock-reference types are always the same type. `Option`, `Collection` and
   `Map` wrap `ValueType::ClockReference` as they wrap any other leaf type, and
   `MapType::new` accepts it as a leaf key type as it accepts `Reference`. The
   language's refusal of a `Reference` map key (FSpec FR-415 "Clock
   collections") is the caller's check.
2. **Payload.** A `ClockPlaceReference` holds the `UniverseId` of the owner's
   universe and the place id text. The universe is the outer universe that
   FSL FR-230 keeps beside the place id and checks against the authenticated
   owner context; the text holds no universe. The value carries no object
   type, no clock value and no valuation.
3. **Canonical encoding check.** `ClockPlaceReference::new(universe, text)`
   constructs a value exactly when `text` meets every rule below. These are
   the rules of FSL FR-230 "Canonical text grammar" that need no declared
   model:
   1. `text` is `place/v2/` followed by exactly five nonempty components
      separated by single `/` characters, with no other `/` and nothing after
      the fifth component. In order the components are `H(P)`, `H(K)`,
      `H(S)`, `H(T)` and `H(J)`.
   2. Each component consists only of the characters `0`-`9` and `a`-`f` and
      has an even length.
   3. The octets each component encodes are UTF-8 and form one JSON value
      that is its own RFC 8785 (JCS) serialization, under the JsonObject
      payload rule of FR-385 Behavior 2.
   4. `K` is a JSON string of at least one character. `P`, `S`, `T` and `J`
      are JSON arrays, and `T` is not empty.
   5. Each entry of `J` is an array of exactly three JSON strings: `"slot"`,
      then `pathPosition`, then `index`. `pathPosition` and `index` match
      `0 | [1-9][0-9]*`. `J` has exactly one entry for each entry of `T`
      that is an array whose first element is the string `"collection"`, in
      the order of those entries in `T`, and each entry's `pathPosition` is
      the decimal spelling of that `T` entry's zero-based position in `T`.

   Any other text returns `ConstructionRefusal { component: Component::Value,
   cause: ConstructionCause::PlaceIdNoncanonical }`, with no value, no
   normalization or repair, and no `CheckedInvariant` fault. A text that
   passes is retained byte for byte, so the canonical text the kernel returns
   equals the text the caller supplied. The caller owns the rules that need
   the declared model: the scope shapes of `P` and `S`, the typed path entry
   and carrier vocabulary of `T`, that the final leaf is Clock, that the scope
   is authenticated, that the owner exists in the admitted population, that
   each slot index is valid for the declared template, and that no place lies
   under a `Map` value (FSL FR-230, FSL FR-106).
4. **Read access.** A `ClockPlaceReference` returns its universe and its place
   id text. Constructing a value from those two outputs gives a value equal to
   the original.
5. **Equality by place.** Two clock-place references in one universe follow
   the existing equality-plan schedule as one terminal pair that contributes
   one `equality.pair` event. They are equal exactly when their place id
   texts are byte equal (FSpec FR-829, FSL FR-230). Two clock-place references
   in different universes return `Refusal::ForeignReference`
   (`foreign_reference`/`foreign-universe`), as two object references in
   different universes do (FR-096-AC-8), because a place id names a place
   only within its owner's universe. A direct plan that pairs a clock-place
   reference with any other value kind, an object reference included, returns
   `CheckedInvariantCause::ValueKindMismatch`.
6. **Canonical key.** `compare_keys` orders two clock-place references by
   universe first, in the order that `UniverseId` gives object references,
   then by the place id texts as unsigned octet sequences, with a proper
   prefix first. The separator `/` (0x2F) sorts below every lowercase
   hexadecimal digit, and lowercase hexadecimal preserves octet order and
   proper prefixes. So within one universe this order is the lexicographic
   order over the five components `P`, `K`, `S`, `T` and `J`, taken in that
   order, each compared as the octets of its canonical JSON with a proper
   prefix first. Two values have equal keys exactly when equality says they
   are equal. A comparison between a clock-place reference and another kind
   returns `None`. The key serves canonical collection order and state keys
   only. It authorizes no ordering operator: the language refuses `<` over
   two place ids (FSpec FR-829, FSL FR-230).
7. **Hash and identity.** `ClockPlaceReference` equality, hashing and ordering
   as a Rust value are those of the pair (universe, place id octets). They
   agree with Behavior 5 within one universe and with Behavior 6 in all
   cases.
8. **Size accounting.** A clock-place reference counts one occurrence (`occ`
   is one, as for an object reference). Its variable size is the byte length
   of its place id text, charged through the semantic-size path that FR-385
   Behavior 6 uses for a JsonObject canonical text. The universe adds no
   variable size.
9. **No intrinsic domain.** The kernel does not enumerate
   `ValueType::ClockReference`: its finite domain is the set of clock places
   in scope, which only the caller's declared model knows. The caller builds
   that domain (FSL FR-106 clock-reference domain) as values made by this
   constructor. Every identity that FSL FR-230 encodes for a declared place
   passes Behavior 3, and Behavior 6 orders the domain deterministically.
10. **Operations.** Equality, key comparison, admission, occurrence counting
    and size accounting are defined on a clock-place reference. The kernel
    defines no arithmetic, conversion, ordering operator or field access on
    it.
11. **Unchanged kinds.** Admission, equality, key comparison and accounting
    for every other `ValueType` and `Value` variant keep their behavior. In
    particular `ValueType::Reference(EffectiveId)` and `ObjectReference` keep
    their object-reference meaning.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-415-AC-1 | A clock-place reference is admitted by `ValueType::ClockReference` and by no other type: `ValueType::Reference(e)`, `Text`, `String`, `Bytes` and `JsonObject` each reject it. `ValueType::ClockReference` rejects an object `Value::Reference` of the same universe whose object identity is the same place id text, and rejects Text, String, Bytes and JsonObject values that hold that text. Two `ValueType::ClockReference` values are equal types. `Option<ClockReference>`, `Sequence<ClockReference>[0, 3]` and `Map<String, ClockReference>` build and admit values whose payload, elements or map values are clock-place references. `MapType::new` with key type `ClockReference` builds a map type. | Test (TC-963) |
| FR-415-AC-2 | `ClockPlaceReference::new` builds a value from the literal `place/v2` text of FSL FR-230-AC-43, and from the same text with `J` replaced by the hex of `[["slot","1","7"]]` where `T` is a field entry followed by one `collection` entry (the slot form of FSL FR-106-AC-81). It returns `PlaceIdNoncanonical`, with no value and no `CheckedInvariant` fault, for: the AC-43 text with one hex letter in uppercase; with `place/v1/` as prefix; with four components; with a trailing `/`; with an empty component; with an odd-length or non-hex component; with `K` the hex of `""`; with `K` the hex of a JSON number `1`; with `T` the hex of `[]`; with `J` the hex of `[["slot","0","0"]]` (a slot entry where `T` has no `collection` entry); with `J` the hex of `[["slot","01","0"]]`; with `J` the hex of `[ ]` (whitespace, not JCS); and with a component whose octets are not UTF-8. | Test (TC-963) |
| FR-415-AC-3 | For each text AC-2 accepts, the value's text output equals the supplied text byte for byte, its universe output equals the supplied universe, and `ClockPlaceReference::new` over those two outputs gives a value whose equality with the original is `true` and whose key is `Equal`. | Test (TC-963) |
| FR-415-AC-4 | In one universe, two values of the AC-43 text are equal under the one-terminal-pair schedule. The AC-43 text against the same text with `K` = `"c/e"`, or with the `field` member URI `ix://i/C/y`, or with a different `P` population URI, is `false`. The planned pair count and charges equal the Boolean-leaf schedule. Two values of the AC-43 text in different universes return `Refusal::ForeignReference` with code `foreign_reference` and cause `foreign-universe`, and no completed Boolean. A direct plan pairing a clock-place reference with an object `Value::Reference`, a Text or a String returns `CheckedInvariant { cause: ValueKindMismatch }`. | Test (TC-963) |
| FR-415-AC-5 | In one universe, `compare_keys` orders `K` = `"c"` before `K` = `"c/d"` (proper prefix of the JSON octets) and `"c/d"` before `"c/e"`, with the other components equal. It orders two values that differ in `P` by `P` before considering `K`. It orders by universe first when universes differ. Equal texts in one universe have key `Equal`. A clock-place reference against an object reference returns `None`. The kernel exposes no ordering operator over clock-place references. A `Set<ClockReference>` built from these values in any insertion order holds them in this key order. | Test (TC-963) |
| FR-415-AC-6 | A clock-place reference has `occ` one and a variable size equal to the byte length of its place id text, so the AC-43 text accounts 279 bytes. Existing object-reference, scalar, composite, Bytes, JsonObject, UUID and Timestamp admission, equality, key and accounting vectors keep their results. | Test (TC-963) |

## Dependencies

- FSL `ix://agent-ix/filament-spec-language/FR-230` "Place identity" and
  "Canonical text grammar" define the place id text, its components and the
  model-dependent rules the caller checks. Its domain-package wire form, the
  FSpec FR-181 reference value whose `identity` is the lowercase hex of the
  place id text, is decoded by FSL to the text this constructor takes.
- FSL `ix://agent-ix/filament-spec-language/FR-106` builds the finite
  clock-reference domain for model checking from values of this type.
- FSpec `ix://agent-ix/filament-specification/FR-829` and
  `ix://agent-ix/filament-specification/FR-415` define `Reference<Clock>` as
  data compared by place, its one `scalar_type`/`clock_reference` node and
  its equality-only operation family. FSpec
  `ix://agent-ix/filament-specification/FR-322` types it.
- FR-385 supplies the JCS canonical-JSON rule, the constructor-refusal pattern
  and the semantic-size path. FR-096 catalogues `foreign_reference`.
- Contract IR FR-352 projects `MemberType::ClockReference` to
  `ValueType::ClockReference`.
