---
id: TC-959
title: "Undefined::NoElement carries the FSpec reason no-element"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/filament-exact/FR-411
    type: verifies
---
# TC-959: Undefined::NoElement carries the FSpec reason no-element

## Description

Verify FR-411-AC-1 through FR-411-AC-3: the public unit variant `Undefined::NoElement`, its distinctness from every other undefined reason, the outcome it forms, and by inspection that no kernel function produces it, that it has no payload and no string spelling, and that one variant covers every unmet index obligation.

## Test Procedure

1. For AC-1, compare `Undefined::NoElement` with each of `DivisionByZero`, `IeeeNotFinite`, `EmptyReduction`, `NoneValue` and `SumOutOfDomain` and with its own clone, and form `Outcome::<i32>::Undefined(Undefined::NoElement)`, reading `completed()` and its kind.
2. For AC-2 and AC-3, inspect the declaration of `Undefined` in `src/outcome.rs` and every use of `NoElement` under `src/`.

## Expected Results

1. `NoElement` is unequal to each other variant and equal to its clone; the outcome is undefined, with no completed value, and is neither refused nor incomplete.
2. `NoElement` is a field-free variant whose doc comment names `no-element` and direct kernel evaluation of an unlinked expression; no function under `src/` returns it, no string spelling of it exists, and `Undefined` has no other index-read variant.
