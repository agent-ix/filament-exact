---
id: SR-16705
title: "Code review of filament-exact PR #195: clock-place reference kernel value"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-exact@f7ced1d7aa8965ed8ca001ed6114d28d6d7d0428; PR #195 diff against origin/main e29f99fd: src/clock_reference.rs, src/equality.rs, src/key.rs, src/lib.rs, src/value.rs, src/value/value_type.rs"
review_set: subset
---
# Code review of filament-exact PR #195

## Summary

Ticket: QSL-2776 (step 1). PR: filament-exact#195, head f7ced1d7. The Rust
lane (rust-review) is folded into this file. Governing spec: FR-415 and
TC-963 at origin/main e29f99fd; the encoding rules were checked against FSL
FR-230 "Canonical text grammar" at filament-spec-language origin/main.

Focused evidence on titan: `cargo test --locked --features
json-object,test-support --lib clock_reference -- --test-threads=3`: 15
passed. `cargo clippy --locked --lib --features json-object -- -D warnings`:
clean. `cargo check --locked --workspace --all-targets` (no features): clean.

Five encoding rules against FR-415 Behavior 3 and FSL FR-230:

1. Prefix `place/v2/` plus `split('/')` into exactly five parts; empty parts
   refused in `decode_component`. Exact.
2. Lowercase `0-9a-f`, nonempty, even length. Exact (uppercase refused).
3. `String::from_utf8`, then `JsonObject::from_json_text` and byte equality
   with `canonical_text()`, i.e. the FR-385 Behavior 2 rule. Exact.
4. Destructure `(K, P, S, T, J)` as `(String, Array, Array, Array, Array)`,
   `K` nonempty, `T` nonempty. Exact.
5. One slot per `T` entry whose first element is the string `"collection"`,
   in order, each `[String "slot", String pos, String idx]`, both decimal
   spellings, `pos == position.to_string()`, then `J` exhausted. Exact.

No rule is missed and none is over-strict: every FR-230 place id satisfies all
five, and nothing outside the five is checked (scope shapes, `T` tags,
carriers are left to the caller as Behavior 3 says). Each rule has a
discriminating adverse test (for example `[ ]` and an unsorted object in `T`
fail only rule 3).

Equality arm: text compared first, `false` on differing text in any universe;
equal text under different universes returns `ForeignReference` with
`required` = left, `supplied` = right, matching the object-reference arm; one
terminal pair. Mixed kinds fall to `ValueKindMismatch`; `compare_keys` mixed
pair falls to `None`. `ClockPlaceReference` derives `Eq/Hash/Ord` over
`(universe, text)` in field order, which is exactly the Behavior 6/7 order
(`String` orders by octets, proper prefix first), and derived equality is
true exactly when the plan returns `true`. `occ` is one through the leaf arm.

Invariant bypass: fields are private, no `Default`, `Deserialize` or other
public constructor; without `json-object` the type has no constructor at all
and is uninhabitable outside the crate. Gating on `json-object` is
acceptable: it follows the `JsonObject::from_json_text` precedent, FSL enables
`json-object` workspace-wide, and contract-ir projects only the type, which is
ungated.

Untrusted input: no indexing, no `unwrap`, no arithmetic overflow
(`hex << 4` is at most 240). Work is linear in the text length; JSON nesting
is bounded by serde_json's default recursion limit of 128 in both parses, so
no stack exhaustion. There is no size cap on the text, the same as
`JsonObject`. Each component is parsed twice (canonicalizer, then
`serde_json::Value`); linear, acceptable.

Size: verified the coder's claim. The kernel has no per-value variable-size
function for any kind (Bytes and JsonObject included); semantic size is
charged by callers through `Charge::size(kind, amount)`. `text().len()` is
the amount a caller charges, as `canonical_text().len()` is for JsonObject.
See the gap-analysis SR for the test-strength note.

Known open (not counted): FR-415-AC-1 says a `Map<ClockReference, Integer>`
built from `A(K="c/e")` then `A(K="c/d")` holds `"c/d"` first. FR-410
Behavior 1 and FR-410-AC-1 say `MapValue::entries()` keeps the supplied order,
and `from_admitted_map` (src/collection.rs:312-382) sorts only a local index
for duplicate detection and stores `entries` unsorted. The conflict is real;
the test asserts only that the map holds two entries. Arch to rule.

Downstream note: `Value`, `ValueType` and `ConstructionCause` are not
`#[non_exhaustive]`, so bumping filament-exact in FSL and contract-ir needs new
match arms there. This is the planned later steps of QSL-2776, not a defect
here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The README feature table says `json-object` adds `JsonObject::from_json_text` only; it now also gates `ClockPlaceReference::new`, and the `ClockPlaceReference` type doc does not say the constructor needs that feature. A consumer without the feature sees a public type it cannot build. | README.md:108 |

## Verdict

Correct and mergeable. The five rules match FR-415 Behavior 3 and FSL FR-230
exactly, equality, key order, derived traits and occurrence count match
FR-415 Behaviors 5-8, and there is no panic path or invariant bypass. FND-001
is a doc nit.
