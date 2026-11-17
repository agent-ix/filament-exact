---
id: SR-200
title: "IR-1558 PR #173 code review — filament-exact map values"
type: SpecReview
analysis: code-review
date: 2026-11-17
scope: "agent-ix/filament-exact PR #173 at 798c35e5bbd7e80598d286ac5f462e943d00a314; IR-1558; src/collection.rs, src/equality.rs, src/key.rs, src/lib.rs, src/value.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/filament-exact/FR-386
    type: reviews
---

## Summary

Reviewed the frozen PR diff in the five changed Rust files, with the Rust style and safety checklist applied. The map value admission and entry-order-independent equality paths are real implementations; map equality's nested key scan is quadratic for unbounded maps, and the ticket's run-form serialization requirements remain outside this PR.

## Verdict

**CONDITIONAL** — one medium resource-cost finding remains in map equality.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Order-independent map equality performs an unbounded nested scan of both entry lists, so comparing valid maps with N distinct keys can require O(N²) key comparisons before value comparison. | src/equality.rs:329-337 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| IR-1558 done-when | examined | Done when Value::Map and its ValueType exist, with canonical key-byte ordering, equality and a round-trip encoding; a tag is cut; and FSL can bump its pin. |
| FR-386-AC-1..9 | context_only | Existing map type identity and admission criteria; the PR adds the value behavior beyond this map-type requirement. |
| src/collection.rs | examined | Adds MapValue, caller-admitted map construction, retained input-order entries, and iterative child dropping. |
| src/equality.rs | examined | Adds map equality by finding a matching right-side key for each left-side key, then compares matching keys and values. |
| src/key.rs | examined | Adds Value::Map to the non-key value arm; maps do not become canonical keys. |
| src/lib.rs | examined | Re-exports MapValue and from_admitted_map. |
| src/value.rs | examined | Adds Value::Map, exact MapType admission, occurrence accounting, iterative drop and debug support. |
| src/value/value_type.rs | context_only | Existing iterative ValueType structural traits already include MapType links. |
