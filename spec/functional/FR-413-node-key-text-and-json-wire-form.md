---
id: FR-413
title: "Node key text and JSON wire form"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-389
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-531
    type: references
  - target: ix://agent-ix/filament-spec-language/FR-070
    type: references
---
# FR-413: Node key text and JSON wire form

## Description

The `filament-exact` kernel SHALL provide the module `filament_exact::node_wire`, behind the optional cargo feature `json`, which converts a structural `NodeKey` to and from its JSON object form `{"package": string, "path": [string | node-identity object, ...], "ordinal": u64}` and to and from the compact JSON text of that object. The module is the one shared codec for the form, and it is `no_std` plus `alloc`. Every conversion walks nested path segments with an explicit stack and reserves its own storage fallibly, so nesting depth is bounded only by memory and no input makes it recurse or abort on allocation failure.

The wire form is the form the language specification already defines (filament-spec-language FR-531 for the shape of an original producer key, and FR-070-AC-12 for its compact spelling). This requirement relocates that form to the kernel, which owns `NodeKey`, so that every consumer shares one codec. It changes no spelling, member order, refusal or bound. The language requirement stays normative for where the form appears in a language artifact and for the nesting limit a language reader applies; this requirement is normative for the codec.

## Use case

A package writer holds a `NodeKey` and needs its text for a `record_key` member, and a reader holds that text and needs the key back. Both call this module instead of each writing a tokenizer, so the writer's bytes and the reader's refusals are the same wherever the key travels. A reader fed untrusted text under a work and storage budget calls the budgeted reader and learns whether a refusal was malformed text, an inadmissible identity, a native allocation failure or the caller's own denial.

## Inputs

- A `&NodeKey` to write, as a JSON value or as text.
- A `serde_json::Value` or a `&str` of JSON text to read.
- For the budgeted text reader, a `&mut dyn IntegerDecimalBudget` (`work` and `reserve`, the caller's admission of traversal work and storage).

## Outputs

- `node_key_to_json(&NodeKey) -> Result<Value, NodeWireError>` and `node_key_from_json(&Value) -> Result<NodeKey, NodeWireError>`.
- `node_key_to_text(&NodeKey) -> Result<String, NodeWireError>`, `node_key_from_text(&str) -> Result<NodeKey, NodeWireError>` and `node_key_from_text_with_budget(&str, &mut dyn IntegerDecimalBudget) -> Result<NodeKey, NodeWireError>`.
- `discard_node_json(Value)`, which releases a `Value` built by `node_key_to_json` without recursing, because dropping a `serde_json::Value` recurses in proportion to depth.
- `NodeWireError`, a `Clone + Copy + Debug + Eq + PartialEq` error with the variants `Shape`, `Package`, `Path`, `Ordinal`, `Allocation`, `Denied` and `Key(InvalidNodeKey)`.

## Behavior

1. **Feature and profile.** `node_wire` SHALL exist only with the cargo feature `json`, which enables `serde_json` without its `std` feature and is off by default; a build without the feature carries no JSON dependency. The module SHALL use only `core` and `alloc`.
2. **JSON object form.** A node identity SHALL be the JSON object with exactly the members `package` (a string), `path` (an array whose elements are each a string, which is a text segment, or a node identity object, which is a nested node segment) and `ordinal` (an unsigned integer). The writer SHALL emit nested node segments as nested identity objects, in path order, to any depth.
3. **Compact text form.** `node_key_to_text` SHALL write the compact JSON text of that object with no whitespace and with the members in the order `ordinal`, `package`, `path`, the order `serde_json` writes an object, at every nesting level. Strings SHALL be written as `serde_json` writes them: quoted, with `"` and `\` escaped, the control characters U+0008, U+000C, U+000A, U+000D and U+0009 written as `\b`, `\f`, `\n`, `\r` and `\t`, any other control character below U+0020 written as `\u00xx`, and every other scalar (including `/` and non-ASCII) written literally. For a key `k`, the text SHALL equal `serde_json::to_string(&node_key_to_json(k))` for every key whose JSON object `serde_json` orders the same way.
4. **Round trip.** For every `NodeKey` `k`, `node_key_from_json(&node_key_to_json(k))` and `node_key_from_text(&node_key_to_text(k))` SHALL return a key equal to `k`, including an empty text segment, a nested node segment at any depth, and the ordinal bounds 0 and `MAX_NODE_ORDINAL`.
5. **Text reader grammar.** `node_key_from_text` and `node_key_from_text_with_budget` SHALL read one identity object in one pass with a hand-written tokenizer for the fixed shape, at cost linear in the text, with an explicit stack. The reader SHALL accept the members in any order, JSON whitespace (space, tab, line feed, carriage return) between tokens and around the whole object, and every JSON string escape (`\" \\ \/ \b \f \n \r \t` and `\uXXXX`, with a surrogate pair combined into one scalar). It SHALL refuse a raw control character below U+0020 in a string, a lone or mismatched surrogate escape, an unknown escape, and any text after the object.
6. **Ordinal in text.** The text reader SHALL read `ordinal` as one or more ASCII decimal digits with no sign, no fraction, no exponent and no leading zero other than a lone `0`, and SHALL refuse a value above `u64::MAX` as `Ordinal`.
7. **Refusals of the JSON reader.** `node_key_from_json` SHALL refuse: a value that is not an object, an object with other than exactly three members, or an object missing `package`, `path` or `ordinal` as `Shape`; a `package` that is not a string as `Package`; a `path` that is not an array, or a path element that is neither a string nor an object, as `Path`; an `ordinal` that is not an unsigned 64-bit integer (negative, fractional, or too large) as `Ordinal`. A nested object is read by the same rules, and its refusal is the refusal of the whole read.
8. **Refusals of the text reader.** The text reader SHALL refuse: text that is not one object of the form above, a missing, duplicate or unknown member, a missing or misplaced separator, an unterminated or malformed string where no member is being read, a `path` not opened by `[`, and trailing content as `Shape`; a `package` value that is not a well-formed string as `Package`; a path element that is neither a well-formed string nor an object as `Path`; an `ordinal` that is absent of digits, has a leading zero or does not fit `u64` as `Ordinal`.
9. **Identity refusal.** When the members are well shaped but name no admissible identity (an empty package, an empty path, or an ordinal above `MAX_NODE_ORDINAL`), every reader SHALL return `Key(InvalidNodeKey)` carrying the kernel's own refusal from `NodeKey::try_new` (FR-389), and no key.
10. **Allocation.** Every allocation the codec reserves explicitly (each output string, vector, path stack and output buffer) SHALL be reserved with `try_reserve` before it is written, and a failed reservation SHALL return `Allocation` with no partial result. Insertion into a `serde_json::Map` node is infallible, because `serde_json` exposes no fallible reservation, and so is the work stack of `discard_node_json`, because teardown returns `()` and cannot report an error; both follow the global allocator's failure behavior, and the public `serde_json::Value` API is unchanged. `node_key_to_text` SHALL return `Shape` only when a string cannot be written.
11. **Budgeted reader.** `node_key_from_text_with_budget` SHALL call the caller's budget, and SHALL map any refusal the budget returns to `Denied` without changing its cause for the caller (a caller's cancellation or deadline is read from the caller's own state). It SHALL charge `work` for the whole text length before it allocates anything, `work(1)` for every parser step, `work` for the byte length of each string it reads, and `work` for the combined package and segment size of each identity before constructing its key; and it SHALL call `reserve` with the minimum bytes of each string, stack and path growth (a vector at capacity SHALL request its doubled capacity, at least four elements, times the element size) before attempting that reservation, and for every reservation `NodeKey::try_new_with` performs. A budget refusal while constructing a key SHALL be `Denied`, distinct from `Key` and `Allocation`. A budget that grants everything SHALL give the same result as `node_key_from_text`, which SHALL be the budgeted reader under an unbounded budget.
12. **Depth.** None of the six functions SHALL recurse in proportion to key depth, and none SHALL impose a depth limit of its own. A key nested 10,000 levels SHALL round trip through the JSON form and through the text form.
13. **Scope.** The projection of a key into a canonical writer or a borrowed-text identity view is not part of this module.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-413-AC-1 | `node_key_to_json` of the key with package `lib`, path `["a"]` and ordinal 2 is the object `{"package":"lib","path":["a"],"ordinal":2}` and carries no other member; a key with a nested node segment is written as a nested object inside the path, in path order. | Test (TC-961) |
| FR-413-AC-2 | `node_key_to_text` of a key nested three levels equals `serde_json::to_string` of its JSON object, writes `ordinal`, `package`, `path` in that order at every level with no whitespace, and writes `"`, `\`, the five short control escapes, a `\u00xx` control and a literal `/` and non-ASCII scalar as stated in Behavior 3. | Test (TC-961) |
| FR-413-AC-3 | A nested key (a node segment among text segments, including an empty text segment) round trips through `node_key_to_json` and `node_key_from_json`, and a key nested 200 levels and one nested 2,000 levels round trip through `node_key_to_text` and `node_key_from_text`, each equal to the original. | Test (TC-961) |
| FR-413-AC-4 | A key nested 10,000 levels round trips through the JSON form with no stack growth and the `Value` is released by `discard_node_json`. | Test (TC-961) |
| FR-413-AC-5 | `node_key_from_json` refuses `"x"`, a missing `ordinal` and an extra member as `Shape`; a numeric `package` as `Package`; a string `path` and a numeric path element as `Path`; ordinal `-1` as `Ordinal`. | Test (TC-961) |
| FR-413-AC-6 | `node_key_from_json` of an empty `package` is `Key(InvalidNodeKey::EmptyPackage)` and of an empty `path` is `Key(InvalidNodeKey::EmptyPath)`, with no key. | Test (TC-961) |
| FR-413-AC-7 | `node_key_from_text` refuses the empty text, `{}`, a text without `path`, an extra member, an `ordinal` written `01`, a numeric path element, content after the object and a repeated `ordinal` member, each with an error and no key; it accepts the members in the order `path`, `package`, `ordinal` with whitespace around the object and its tokens. | Test (TC-961) |
| FR-413-AC-8 | `node_key_from_text` reads the escapes `k`, the surrogate pair `😀`, and `\" \\ \b \f \n \r \t \/` to the same key `node_key_from_json` returns for the same document, and refuses a lone `\ud800`, an unknown escape and a raw control character in a string as an error with no key. | Test (TC-961) |
| FR-413-AC-9 | Under a recording budget, `node_key_from_text_with_budget` of a key nested 1,024 levels returns the key when the budget grants everything; with a budget whose total is one unit short of what that read consumed it returns `Denied`, and its last `reserve` request equals the full read's last request. | Test (TC-961) |
| FR-413-AC-10 | A budget whose caller state is cancelled (requested, and deadline) makes `node_key_from_text_with_budget` return `Denied`, the caller's state reads the same cause afterwards, and the budget has received no `reserve` request, because the work for the whole text is charged before any parser allocation. | Test (TC-961) |
| FR-413-AC-11 | Every explicit reservation of the codec is tested: a budget that denies a `reserve` request makes `node_key_from_text_with_budget` return `Allocation` with no key, and a failed reservation of an output string, vector, path stack or output buffer returns `Allocation` with no partial result. Insertion into a `serde_json::Map` node is not tested for failure. | Test (TC-961) |

## Status

Specified, not yet implemented on main. The code and its tests move from `fsl-foundation::node_wire`, which then re-exports or calls this module.

## Dependencies

- [FR-389](./FR-389-carry-admitted-structural-identities.md) owns `NodeKey`, `InvalidNodeKey`, `NodeKey::try_new`, the policy-aware constructor and `MAX_NODE_ORDINAL`; this requirement adds only the wire form.
- filament-spec-language `ix://agent-ix/filament-spec-language/FR-531` specifies the `record_key` shape this codec reads and writes and the nesting limit a language reader applies; `ix://agent-ix/filament-spec-language/FR-070` specifies the compact spelling in FR-070-AC-12. Those requirements stay normative for the language and point here for the codec.
- The falsifying controls are [TC-961](../test-cases/TC-961-node-key-text-and-json-wire-form.md).
