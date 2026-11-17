---
id: SR-201
title: "IR-1558 PR #173 gap analysis — filament-exact map values"
type: SpecReview
analysis: gap-analysis
date: 2026-11-17
scope: "agent-ix/filament-exact PR #173 at 798c35e5bbd7e80598d286ac5f462e943d00a314; IR-1558; src/collection.rs, src/equality.rs, src/key.rs, src/lib.rs, src/value.rs, spec/functional/FR-386-map-value-type-with-a-key-type.md, spec/test-cases/TC-935-map-value-type-with-a-key-type.md"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-386
    type: reviews
  - target: ix://agent-ix/filament-exact/TC-935
    type: reviews
---

## Summary

Planless gap analysis of the full repository matrix plus PR #173's changed code and the IR-1558 acceptance. FR-386-AC-1 through AC-9 remain statically tagged by TC-935 tests; the new map-value behavior is not owned by a requirement in this repository, and the ticket's canonical run-form ordering and round-trip encoding are not delivered in this PR. Plan completion: not assessed.

## Verdict

**FAIL** — quoin matrix reports 116 untagged criteria repository-wide; the PR also introduces map-value behavior with no local owning requirement and leaves two ticket acceptance items unimplemented.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The computed repository matrix reports 116 untagged criteria out of 422, so those criteria have no trace-tagged test binding. This baseline gap is outside the PR's changed area. | quoin matrix --repo . --json |
| FND-002 | medium | The PR adds MapValue construction and equality, but FR-386 explicitly excludes map values and no later map-value requirement exists in this repository; the new test instead names external FR-650-AC-11, which the local matrix cannot resolve. | src/collection.rs:312-316; src/equality.rs:325-358; spec/functional/FR-386-map-value-type-with-a-key-type.md:Status |
| FND-003 | medium | IR-1558 also requires canonical key-byte run-form ordering and round-trip encoding. from_admitted_map retains caller order and delegates canonical output ordering to its caller; this PR contains no map encoder or round-trip test. The issue notes FSL owns the FR-650 run form, so these acceptance items remain for that consumer handoff. | src/collection.rs:312-315; IR-1558 |

## Coverage

- Matrix: quoin 0.35.0 reports 422 criteria: 262 tagged, 116 untagged and 44 without a test symbol; all 422 have no run evidence. FR-386-AC-1 through AC-9 each remain tagged by one or more TC-935 test functions. There is no FR-650 requirement in this repository, so the added FR-650-AC-11 tag is not represented in the local matrix.
- Code to requirement: the new map value representation, constructor, admission and equality have no local owning functional requirement. FR-386's Status says map values are outside its scope and would need a later requirement.
- Stubs and coverage inflation: none found in the changed implementation or the new map equality test. The new test uses real Value and ValueType paths and asserts entry-order-independent equality, unequal values, exact map-type admission and retained input order.
- Semantic review: skipped; it is optional and was not requested.
- Plan completion: not assessed.
- Toolchain limitation: this run resolved quoin 0.35.0, quire 0.49.0 (engine 0.62.0), and spec-artifacts-process@v0.31.0; repository instructions identify quoin 0.30.0 / quire 0.40.0 (engine 0.56.0). The matrix and validation results therefore use the newer installed chain.

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| IR-1558 done-when | examined | Done when Value::Map and its ValueType exist, with canonical key-byte ordering, equality and a round-trip encoding; a tag is cut; and FSL can bump its pin. |
| FR-386-AC-1 | examined | ValueType::map(MapType::new(K, V, bound)?) returns a ValueType::Map whose MapType::key() equals K, whose MapType::values() equals V and whose MapType::bound() equals bound, for bound both None and Some. |
| FR-386-AC-2 | examined | Two map types are equal exactly when key type, value type and bound are equal; equal maps hash equal; a map is unequal to collection and option types. |
| FR-386-AC-3 | examined | A map whose value type is a map, an Option or a Collection compares by its whole nested chain. |
| FR-386-AC-4 | examined | On a 512 KiB stack, deeply nested map ValueTypes clone, compare, hash, format and drop without stack overflow. |
| FR-386-AC-5 | examined | Debug of bounded and unbounded map types prints the specified derived layout. |
| FR-386-AC-6 | examined | ValueType::admits on a Map type returns false for collection, text and option values. |
| FR-386-AC-7 | examined | Alternate Debug of map types prints the specified pretty layout. |
| FR-386-AC-8 | examined | MapType::key returns the declared key type; types differing by key are unequal. |
| FR-386-AC-9 | examined | MapType::new refuses nested key types and retains each supported leaf key type. |
| FR-386 Status | examined | Map values (construction, key uniqueness, canonical order, metered charges) are not part of this requirement; a later requirement adds them if a caller needs a kernel map value. |
| TC-935 | examined | Tests exercise public map-type construction, identity, nested chains, deep type traits, Debug and type admission; no map-value construction or equality is in this test case. |
| src/collection.rs | examined | Adds MapValue, from_admitted_map, retained input-order entries and iterative child dropping. |
| src/equality.rs | examined | Matches each left map key against right-side entries, then schedules matching key/value comparisons. |
| src/key.rs | examined | Excludes Value::Map from canonical-key comparison. |
| src/lib.rs | examined | Exports MapValue and from_admitted_map. |
| src/value.rs | examined | Adds Value::Map and matching ValueType admission, occurrence, drop and debug support. |
| IR-1558 round-form acceptance | examined | Canonical key-byte ordering and round-trip encoding are part of the issue's Done-when text; the issue assigns the FR-650 JCS run form to FSL. |
