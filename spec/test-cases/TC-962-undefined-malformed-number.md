---
id: TC-962
title: "Undefined::MalformedNumber carries the FSpec reason malformed-number"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-414
    type: verifies
---
# TC-962: Undefined::MalformedNumber carries the FSpec reason malformed-number

## Description

Verify FR-414-AC-1 through FR-414-AC-3: the public unit variant `Undefined::MalformedNumber`, its distinctness from every other undefined reason, the outcome it forms, and by inspection that it has no payload and no string spelling, and that one variant covers every text that is not a number.

## Test Procedure

1. For AC-1, compare `Undefined::MalformedNumber` with each of `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue`, `NoElement` and `SumOutOfDomain` and with its own clone, and form `Outcome::<i32>::Undefined(Undefined::MalformedNumber)`, reading `completed()` and its kind.
2. For AC-2 and AC-3, inspect the declaration of `Undefined` in `src/outcome.rs` and every use of `MalformedNumber` under `src/`.

## Expected Results

1. `MalformedNumber` is unequal to each other variant and equal to its clone; the outcome is undefined, with no completed value, and is neither refused nor incomplete.
2. `MalformedNumber` is a field-free variant whose doc comment names `malformed-number` and the text-to-number condition; no string spelling of it exists, and `Undefined` has no other variant for a text that is not a number.
