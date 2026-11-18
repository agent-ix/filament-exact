---
id: FR-414
title: "The kernel carries the undefined reason malformed-number as Undefined::MalformedNumber"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-specification/FR-478
    type: references
---
# FR-414: The kernel carries the undefined reason malformed-number as Undefined::MalformedNumber

## Description

The `filament-exact` kernel SHALL provide `Undefined::MalformedNumber`, the exact carrier of the FSpec undefined reason `malformed-number`: a text-to-number `convert<T>(e)` whose operand is not a number in the production of `T`'s kind, the empty text included (FSpec FR-478 "Number grammar"). It is the value an evaluator returns as `Outcome::Undefined(Undefined::MalformedNumber)` for such a conversion, so the reason has one typed spelling from the evaluator to FSL's outcome writer.

## Use case

A model holds an amount typed into a form field as text, and a claim converts it with `convert<Int[0, 99]>(t)`. The field holds `abc`. After the operand and normalization charges the evaluator returns `Outcome::Undefined(Undefined::MalformedNumber)` located at the `convert` node, and FSL writes the outcome as `undefined` with reason `malformed-number`.

## Inputs

- None. `MalformedNumber` is a unit variant.

## Outputs

- `Undefined::MalformedNumber`, carried by `Outcome::Undefined` and `Stop::Undefined`.

## Behavior

1. **Spelling.** `MalformedNumber` SHALL be a unit variant of the public `Undefined` enum, beside `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue`, `NoElement` and `SumOutOfDomain`, and distinct from each of them under `PartialEq` and `Hash`. Its doc comment names the FSpec reason `malformed-number` and states that it is the outcome of a text-to-number conversion whose text is not a number for its target.
2. **Mapping.** `MalformedNumber` corresponds one to one to the FSpec native-diagnostics undefined reason `malformed-number`: every outcome carrying `MalformedNumber` is written `malformed-number`, and `malformed-number` is read back as `MalformedNumber` and as no other variant. It is the one variant for every target kind (integer, decimal, rational, IEEE and quantity) and every way a text fails its production, because FSpec gives them one reason; the kernel adds no per-target or per-defect variant and no payload, so the outcome never carries the operand text. The kernel holds no string spelling of any `Undefined` variant; the reason's spelling `malformed-number` is FSpec's, and FSL's outcome writer applies it.
3. **Where it is produced.** An evaluator produces it at a `filament.op.text.*` or `filament.op.string.*` conversion node, after the operand charges and any normalization charges and before any target-step charge, with no value (FSpec FR-478 "Accounting"). A text that is a number but lies outside `T` is a refusal, never `MalformedNumber`.
4. **Kind of outcome.** `MalformedNumber` is an undefined outcome. It is never a refusal, an incomplete record or a `CheckedInvariantCause`, and it carries no caller cause (FR-090).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-414-AC-1 | `Undefined::MalformedNumber` is a public unit variant; it is unequal to each of `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue`, `NoElement` and `SumOutOfDomain`, its clone equals it, and `Outcome::<i32>::Undefined(Undefined::MalformedNumber)` is neither completed, refused nor incomplete. | Test (TC-962) |
| FR-414-AC-2 | Inspection of the crate shows that the variant carries no field and that its doc comment names the FSpec reason `malformed-number` and the text-to-number condition. | Inspection (TC-962) |
| FR-414-AC-3 | Inspection of the crate shows no string spelling of `Undefined::MalformedNumber` and no second `Undefined` variant for a text that is not a number, so every target kind and every grammar defect share the one variant. | Inspection (TC-962) |

## Status

Specified, not yet implemented on main.

## Dependencies

- FSpec `ix://agent-ix/filament-specification/FR-478` "Number grammar" and "Value and outcomes" define when a text is not a number and give the undefined reason `malformed-number`; the native-diagnostics catalog lists the reason.
- [FR-090](./FR-090-kernel-outcome-carries-no-caller-cause.md) keeps kernel outcomes free of caller causes.
- The falsifying controls are [TC-962](../test-cases/TC-962-undefined-malformed-number.md).
