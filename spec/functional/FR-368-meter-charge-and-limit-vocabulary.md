---
id: FR-368
title: "Public meter charge and limit vocabularies are complete and uniquely spelled"
type: FR
relationships:
  - target: ix://agent-ix/filament-specification/FR-013
    type: depends_on
---
# FR-368: Public meter charge and limit vocabularies are complete and uniquely spelled

## Description

The `filament-exact` meter SHALL expose a complete, uniquely spelled public `ChargePoint` vocabulary and a `LimitKind` vocabulary in `ScalarLimits` field order.

## Behavior

`ChargePoint::ALL` and `LimitKind::ALL` enumerate their respective public enum variants once. Each `as_str()` returns its documented name. `ChargePoint::from_code()` resolves precisely those spellings and rejects unknown spellings. The public limit-kind order is the order in which a charge tests semantic sizes before cumulative work and results.

`TemporalVisit`, spelled `temporal.visit` by `as_str()`, follows `model.navigate` at index 62 of `ChargePoint::ALL`; the first 62 entries retain their existing order. It names a caller-owned temporal operator visit charged through the existing `WorkUnits` limit, not a new limit kind. It is distinct from the graph-walk points `graph.expand`, `graph.edge` and `graph.result-retain`.

The `ChargePoint` vocabulary SHALL append `FrameWrite` (`frame.write`) at index 63 and `FrameRedefinesLink` (`frame.redefines-link`) at index 64, in that order, without changing the identity or order of entries 0 through 62. These names identify the frame events defined by the authoritative operation-frame and value-accounting contracts. The operation-frame caller SHALL own detection and ordering of field writes, traversal of `redefines` links, and admission into its own frame meter. This vocabulary extension SHALL NOT move frame checking into `filament-exact`, merge the caller's `ModelNormalizationLimits.work_units` with the kernel's `ScalarLimits.work_units`, or add a kernel-owned frame charge schedule. A caller that directly uses the kernel `Meter` remains subject to that meter's ordinary named-point behavior; this does not alter the independent caller meter.

`ModelCandidate`, spelled `model.candidate`, names caller-owned candidate generation, including argument vectors, candidate states and result pairs. A batch requests its actual count through ordinary work charging. It is appended at index 65; earlier point identities and order remain unchanged.

`ModelCandidate` also admits canonical plan policy key work and capacity retention through two explicit alternatives, with no semantic-size requests:

- `Charge::new(ChargePoint::ModelCandidate).work(Integer::from(N))` charges `WorkUnits += N` and zero result units for actual node and byte visits, before traversal, encoding, decoding, comparison or copying.
- `Charge::new(ChargePoint::ModelCandidate).work(Integer::zero()).results(B)` charges zero work and `ResultUnits += B` before each storage growth. `B` is the full new requested capacity in bytes, computed with checked arithmetic, including scratch, output and owned key copies; it is neither a capacity delta nor a typed-value occurrence count. Successful reservations accumulate without refunds.

Each search, replay or checker session SHALL use its established evaluation ledger and original `Cancel` for these charges, preserving ordinary atomic admission and first-denial behavior. Per-key or per-instance meters SHALL NOT reset either allowance. `WorkScope` shares work only and does not share the result ledger. Existing candidate-count charging, model-only `max_state_key_bytes` and `IdentityLimits` remain unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-368-AC-1 | Every public `ChargePoint` variant occurs exactly once in `ChargePoint::ALL`, has a unique documented spelling from `as_str()` and round-trips through `from_code()`; an unknown spelling returns `None`. `TemporalVisit` spells `temporal.visit` and is at index 62 after `ModelNavigate` at index 61; entries 0 through 61 retain their pre-extension order. Exhaustive variant matching makes an added but unaccounted variant fail the check. | Test |
| FR-368-AC-2 | Every public `LimitKind` variant occurs exactly once in `LimitKind::ALL`, has a unique documented `as_str()` spelling, and the sequence is exactly the ten `ScalarLimits` fields: integer bits, decimal digits, scale expansion, text input bytes, text scalars, normalized scalars, unit edges, value occurrences, work units and result units. Exhaustive variant matching makes an added but unaccounted variant fail the check. | Test |
| FR-368-AC-3 | The public typed variants `FrameWrite` and `FrameRedefinesLink` spell exactly `frame.write` and `frame.redefines-link`; each round-trips through `from_code()`, while misspellings return `None`. `ChargePoint::ALL` contains exactly 66 distinct variants, with `TemporalVisit` at index 62, `FrameWrite` at 63 and `FrameRedefinesLink` at 64 and `ModelCandidate` (`model.candidate`) at 65; entries 0 through 62 retain their earlier identities and order. | Test |
| FR-368-AC-4 | Inspection of the public kernel API and the caller's frame path identifies the two typed names in the kernel vocabulary and the caller's independent frame event, meter and `ModelNormalizationLimits.work_units` ownership. No new kernel frame traversal or counter sharing with `ScalarLimits` is introduced. | Inspection |

## Status

PR #9 adds executable Trace bindings for AC-1 and AC-2. Run `quire matrix` for the current criterion-to-test mapping.

`TemporalVisit` and `temporal.visit` remain normative public spellings. AC-3 and AC-4 are planned until the frame vocabulary, executable binding and boundary inspection land.

## Dependencies

- [TC-916](../test-cases/TC-916-meter-charge-and-limit-vocabulary.md) verifies the vocabulary criteria through the public enum APIs. [FR-359](./FR-359-cumulative-meter-boundary.md) owns first-short-counter behavior that uses the limit-kind order. Operation-frame event and amount semantics remain with `ix://agent-ix/filament-specification/FR-013` and its value-accounting definition; FSL owns the independent caller meter.
