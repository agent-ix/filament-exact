---
id: SR-17940486709564847048073
title: QSL-1978 PR192 accounting owner spec-review/scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/filament-exact; src/accounting.rs ModelCandidate doc comment; spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  Behavior and unchanged AC context
review_set: subset
relationships:
- target: ix://agent-ix/filament-exact/FR-368
  type: reviews
---

## Summary

Ticket: QSL-1978. Independent bounded review of PR192 at frozen head e92ff3cec8b11c22c3ca34fbb1181b23cae35682 against base a58474cc105a86f3307a11def3fc001e2459679f. Exact model ID unavailable; native Codex reviewer, no Jev used. No applicable AssuranceProfile exists in the spec tree.

## Verdict

PASS. Exact owns the public ModelCandidate vocabulary and generic Charge/Meter vectors (cross-cutting accounting). Search/replay/checker callers own policy key visits, growth, existing per-session evaluation ledgers and original Cancel. Caller lifecycle is a specified obligation, not runtime evidence here. WorkScope shares only work; replay/checker retain their established independent sessions. IdentityLimits and model-only max_state_key_bytes stay unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

```yaml
scope:
- id: ModelCandidate-definition
  path: src/accounting.rs
  role: examined
  excerpt: "    /// `model.candidate`: caller-owned candidate generation and canonical plan policy key work\n  \
    \  /// and capacity retention.\n    ModelCandidate,"
- id: FR-368-key-alternatives
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: '`ModelCandidate` also admits canonical plan policy key work and capacity retention through two explicit
    alternatives, with no semantic-size requests:'
- id: FR-368-key-work
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: '- `Charge::new(ChargePoint::ModelCandidate).work(Integer::from(N))` charges `WorkUnits += N` and zero
    result units for actual node and byte visits, before traversal, encoding, decoding, comparison or copying.'
- id: FR-368-key-retention
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: '- `Charge::new(ChargePoint::ModelCandidate).work(Integer::zero()).results(B)` charges zero work and
    `ResultUnits += B` before each storage growth. `B` is the full new requested capacity in bytes, computed with
    checked arithmetic, including scratch, output and owned key copies; it is neither a capacity delta nor a typed-value
    occurrence count. Successful reservations accumulate without refunds.'
- id: FR-368-key-lifecycle
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: Each search, replay or checker session SHALL use its established evaluation ledger and original `Cancel`
    for these charges, preserving ordinary atomic admission and first-denial behavior. Per-key or per-instance meters
    SHALL NOT reset either allowance. `WorkScope` shares work only and does not share the result ledger. Existing
    candidate-count charging, model-only `max_state_key_bytes` and `IdentityLimits` remain unchanged.
- id: FR-368-candidate-count
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: '`ModelCandidate`, spelled `model.candidate`, names caller-owned candidate generation, including argument
    vectors, candidate states and result pairs. A batch requests its actual count through ordinary work charging.
    It is appended at index 65; earlier point identities and order remain unchanged.'
- id: FR-368-AC-1
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: context_only
  excerpt: Every public `ChargePoint` variant occurs exactly once in `ChargePoint::ALL`, has a unique documented
    spelling from `as_str()` and round-trips through `from_code()`; an unknown spelling returns `None`. `TemporalVisit`
    spells `temporal.visit` and is at index 62 after `ModelNavigate` at index 61; entries 0 through 61 retain their
    pre-extension order. Exhaustive variant matching makes an added but unaccounted variant fail the check.
- id: FR-368-AC-2
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: context_only
  excerpt: 'Every public `LimitKind` variant occurs exactly once in `LimitKind::ALL`, has a unique documented `as_str()`
    spelling, and the sequence is exactly the ten `ScalarLimits` fields: integer bits, decimal digits, scale expansion,
    text input bytes, text scalars, normalized scalars, unit edges, value occurrences, work units and result units.
    Exhaustive variant matching makes an added but unaccounted variant fail the check.'
- id: FR-368-AC-3
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: context_only
  excerpt: The public typed variants `FrameWrite` and `FrameRedefinesLink` spell exactly `frame.write` and `frame.redefines-link`;
    each round-trips through `from_code()`, while misspellings return `None`. `ChargePoint::ALL` contains exactly
    66 distinct variants, with `TemporalVisit` at index 62, `FrameWrite` at 63 and `FrameRedefinesLink` at 64 and
    `ModelCandidate` (`model.candidate`) at 65; entries 0 through 62 retain their earlier identities and order.
- id: FR-368-AC-4
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: context_only
  excerpt: Inspection of the public kernel API and the caller's frame path identifies the two typed names in the
    kernel vocabulary and the caller's independent frame event, meter and `ModelNormalizationLimits.work_units`
    ownership. No new kernel frame traversal or counter sharing with `ScalarLimits` is introduced.
```

## Evidence and limits

Read actual two-file diff and existing Charge::new/work/results, WorkScope and Meter atomic cumulative admission implementation. Quire exact FR-368 validation exits 0; git diff --check exits 0. Installed quoin 0.36.8; quire 0.52.0 (engine 0.62.2); spec-artifacts-process@v0.36.0. No Cargo, runtime tests, full gates, matrix or baselines run, per documentation-only dispatch. Gap-analysis is inapplicable because no executable production code or test contract changed. Integrity, dependency and object lenses are inapplicable because no structure, relationships or domain objects changed. No criterion-strength/Jev result claimed.
