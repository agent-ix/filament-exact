---
id: FR-411
title: "The kernel carries the undefined reason no-element as Undefined::NoElement"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-specification/FR-144
    type: references
  - target: ix://agent-ix/filament-eval/FR-009
    type: references
---
# FR-411: The kernel carries the undefined reason no-element as Undefined::NoElement

## Description

The `filament-exact` kernel SHALL provide `Undefined::NoElement`, the exact carrier of the FSpec undefined reason `no-element`: an index read `c[i]` whose index obligation is unmet, so the element it names does not exist (QSpec FR-144 "Indexing"). It is the value an evaluator returns as `Outcome::Undefined(Undefined::NoElement)` for such a read, so the reason has one typed spelling from the evaluator to FSL's outcome writer.

## Use case

An evaluator runs an unlinked body directly in the kernel, which proves no index obligation. It meets `s[3]` over a three-element sequence. After both operands complete it returns `Outcome::Undefined(Undefined::NoElement)` located at the index node, charges no `collection.element`, and FSL writes the outcome as `undefined` with reason `no-element`.

## Inputs

- None. `NoElement` is a unit variant.

## Outputs

- `Undefined::NoElement`, carried by `Outcome::Undefined` and `Stop::Undefined`.

## Behavior

1. **Spelling.** `NoElement` SHALL be a unit variant of the public `Undefined` enum, beside `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue` and `SumOutOfDomain`, and distinct from each of them under `PartialEq` and `Hash`. Its doc comment names the FSpec reason `no-element` and states that only direct kernel evaluation of an unlinked expression can meet it, as the `EmptyReduction` and `NoneValue` comments do.
2. **Mapping.** `NoElement` corresponds one to one to the FSpec native-diagnostics undefined reason `no-element`: every outcome carrying `NoElement` is written `no-element`, and `no-element` is read back as `NoElement` and as no other variant. It is the one variant for all three unmet obligations, a sequence index below 0 or at or past the size, an ordered-set index out of range and a map key with no entry, because FSpec gives all three one reason; the kernel adds no per-case variant and no payload. The kernel holds no string spelling of any `Undefined` variant; the reason's spelling `no-element` is FSpec's, and FSL's outcome writer applies it.
3. **Where it is produced.** The kernel has no collection index operation, so no kernel function returns `NoElement`. An evaluator produces it at a collection index node (filament-eval FR-009 "Collection index"), after both operands complete and before the `collection.element` charge, with no value. A linked body has proved the obligation and never produces it.
4. **Kind of outcome.** `NoElement` is an undefined outcome. It is never a refusal, an incomplete record or a `CheckedInvariantCause`, and it carries no caller cause (FR-090).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-411-AC-1 | `Undefined::NoElement` is a public unit variant; it is unequal to each of `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue` and `SumOutOfDomain`, its clone equals it, and `Outcome::<i32>::Undefined(Undefined::NoElement)` is neither completed, refused nor incomplete. | Test (TC-959) |
| FR-411-AC-2 | Inspection of the crate shows that no kernel function returns `Undefined::NoElement`, that the variant carries no field, and that its doc comment names the FSpec reason `no-element` and the direct-kernel-evaluation condition. | Inspection (TC-959) |
| FR-411-AC-3 | Inspection of the crate shows no string spelling of `Undefined::NoElement` and no second `Undefined` variant for an index read, so an out-of-range sequence index, an out-of-range ordered-set index and an absent map key share the one variant. | Inspection (TC-959) |

## Status

Specified, not yet implemented on main.

## Dependencies

- QSpec `ix://agent-ix/filament-specification/FR-144` "Indexing" defines the index obligation and the undefined reason `no-element`; the native-diagnostics catalog lists the reason.
- `ix://agent-ix/filament-eval/FR-009` produces the outcome at its collection index node.
- [FR-090](./FR-090-kernel-outcome-carries-no-caller-cause.md) keeps kernel outcomes free of caller causes.
- The falsifying controls are [TC-959](../test-cases/TC-959-undefined-no-element.md).
