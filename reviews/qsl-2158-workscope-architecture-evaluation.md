---
id: SR-17940486709564847048068
title: architecture-evaluation review of QSL-2158 shared work owner
type: SpecReview
analysis: architecture-evaluation
scope: agent-ix/filament-exact; PR191 QSL-2158; src/accounting.rs, src/accounting/units.rs,
  src/accounting/work_scope_tests.rs, src/lib.rs, tests/ir673_core.rs, FR-359, FR-368,
  AD-002
review_set: subset
relationships:
- target: ix://agent-ix/filament-exact/FR-359
  type: references
- target: ix://agent-ix/filament-exact/FR-368
  type: references
- target: ix://agent-ix/filament-exact/AD-002
  type: references
---

## Summary

The implemented owner seam matches AD-002 and the approved API responsibility split.

Frozen PR191 source reviewed independently against the approved owner contract. Ticket: QSL-2158. Head: d1bcce010f226c75c25b1ed171f02ee981e9ab03; base: b26a8e4c592503fccdc4a0e3d2cce23dd8487616. The reviewed source has eight changed files (412 additions, 19 deletions), no dependency, package version or CI changes. No applicable AssuranceProfile exists under spec/. No product source, specification, branch, tracker or GitHub comments were mutated by this reviewer.

Actual Titan execution at this exact head: cargo check --locked --tests -p filament-exact rc0; default accounting::work_scope_tests 9 passed; test-support same prefix 9 passed; ir673_core charge_point_vocabulary_is_unique_and_round_trips --exact 1 passed (19 total runtime passes, zero failures/ignored). These are the authorized focused receipts, not a broad gate, new Kani proof or downstream integration result. No local Cargo command ran.

Publication remains pending the user's Linear hold and root coordination. No Linear reads or writes occurred. The known has-review label is supplied by the dispatch, not independently queried. Draft YAML is locally checked; tracker dedup/read-back remains pending.

## Verdict

PASS — The implemented owner seam matches AD-002 and the approved API responsibility split.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Success scenario: cloned WorkScope yields fresh ordinary meters; each charge linearizes under the same prefix while retaining local high-water/result counters and local admitted-event capture. Failure scenario: work or local result denial returns original typed fields and commits nothing; cancellation observes the prefix before acquiring the transition lock. Recovery scenario: fresh meter/clone/drop never restores entitlement. Change scenario: ModelCandidate is an appended kernel catalog name; downstream callers retain event scheduling and can pass the ordinary owned Meter through existing ports. Equality reservation checks shared remainder while effects still occur through real meter operations. AD-002's same-walk relation/count and unsupported cross-crate Kani decomposition boundaries are preserved; this paragraph does not qualify those proofs or require a new scheduler. No guard escapes to callbacks/evaluators. Source evidence: accounting.rs WorkScope/with_work_scope/charge_general_checked/charge_plan_checked, units.rs both checked native paths, lib.rs export, work_scope_tests.rs adverse cases. Mutex contention remains an unmeasured performance risk, not a demonstrated defect or invented numeric architecture score.

## Examined Scope

```yaml
scope:
- id: FR-359-AC-13
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: 'Two scoped meters each requesting three units under bound five share the
    same prefix: the second denies with original limit five, consumed three, next
    charge three and its actual point. Cloning a meter or scope and dropping a meter
    do not reset consumption; a later two-unit charge reaches five.'
- id: FR-359-AC-14
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: After another scoped meter spends two units under bound five, both exact
    and native one-pair equality plans admit their three-unit availability reservation
    but commit only one unit. Repeating the plan denies at prefix three with original
    bound five and next charge three.
- id: FR-359-AC-15
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: Common and general charges denied for local size or later result capacity
    leave the shared prefix, local high-water and admissions unchanged. Another fresh
    scoped meter has zero local size and result counters and can use its own result
    entitlement.
- id: FR-359-AC-16
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: Scoped exact work above `u64::MAX` retains the exact denial amount and
    wins before an oversized result. An oversized result after a fitting work request
    commits no work. The final fitting unit reaches `u64::MAX`; the next unit denies
    without overflow.
- id: FR-359-AC-17
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: A cancellation observer can read the same scope without deadlock. Cancellation
    precedes a pending injection, both denied requests preserve shared work and their
    special limit/consumed fields, cancellation records its cause, and injection alone
    does not trip cancellation.
- id: FR-359-AC-18
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: A meter charging two then three units while another scoped meter charges
    four reports five work units in its interval accounting and only its own two events
    in bounded capture; the shared prefix is nine. Concurrent scoped requests cannot
    admit more than the original entitlement.
- id: FR-359-AC-19
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: Plain meter clones initially compare equal and independently consume their
    remaining work; a fresh plain meter starts at zero.
- id: FR-368-AC-1
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: Every public `ChargePoint` variant occurs exactly once in `ChargePoint::ALL`,
    has a unique documented spelling from `as_str()` and round-trips through `from_code()`;
    an unknown spelling returns `None`. `TemporalVisit` spells `temporal.visit` and
    is at index 62 after `ModelNavigate` at index 61; entries 0 through 61 retain
    their pre-extension order. Exhaustive variant matching makes an added but unaccounted
    variant fail the check.
- id: FR-368-AC-3
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: The public typed variants `FrameWrite` and `FrameRedefinesLink` spell exactly
    `frame.write` and `frame.redefines-link`; each round-trips through `from_code()`,
    while misspellings return `None`. `ChargePoint::ALL` contains exactly 66 distinct
    variants, with `TemporalVisit` at index 62, `FrameWrite` at 63 and `FrameRedefinesLink`
    at 64 and `ModelCandidate` (`model.candidate`) at 65; entries 0 through 62 retain
    their earlier identities and order.
- id: FR-359
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: '`WorkScope::new(ScalarLimits)` fixes one work entitlement. Cloned scopes
    and ordinary meters returned by `meter()` share only admitted `WorkUnits`; `consumed_work()`
    and scoped `Meter::consumed(WorkUnits)` observe that shared prefix. Each new scoped
    meter starts every other counter at zero under the original limits. Scope equality
    is handle identity. Meter clones copy local state and retain the same scope; dropping
    or cloning either handle cannot replenish work. Plain `Meter::new` and its value
    equality and independent clones retain their existing behavior.'
- id: FR-359
  path: spec/functional/FR-359-cumulative-meter-boundary.md
  role: examined
  excerpt: 'Scoped common, general/exact and equality-plan charges validate and commit
    under one owner synchronization boundary. Cancellation and its observer are polled
    before entering that boundary; no evaluator or observer callback runs under its
    guard. Size, shared work and local result validation retain existing priority
    and a later denial commits nothing. Equality-plan availability checks the full
    `pairs + 2` reservation against the shared remainder and commits only one work
    unit. Ordinary shared shortages report the original configured bound, the admitted
    shared prefix, the actual next request and '
- id: FR-368
  path: spec/functional/FR-368-meter-charge-and-limit-vocabulary.md
  role: examined
  excerpt: '`ModelCandidate`, spelled `model.candidate`, names caller-owned candidate
    generation, including argument vectors, candidate states and result pairs. A batch
    requests its actual count through ordinary work charging. It is appended at index
    65; earlier point identities and order remain unchanged.'
- id: AD-002
  path: spec/assurance/AD-002-composable-equality-proof-seam.md
  role: examined
  excerpt: ' in `LimitKind` field order, then cumulative work, then cumulative results,
    and admits nothing on refusal. `charge_plan` checks an injected named denial first,
    then pair-count `ValueOccurrences`, work reservation, and result capacity before
    mutating the meter. With a `WorkScope`, this same transition uses the shared work
    prefix and original bound while sizes and results stay local; the full work reservation
    is checked atomically with local result capacity, and only the plan''s existing
    one-unit increment is committed. The scope supplies atomic accounting rather than
    worker scheduling.'
```

## Tracker publication

Published privately to QSL-2158 after the user hold and explicit root release. No GitHub findings, label or issue-state changes were made. Earlier pending-publication statements describe the historical draft state.

- Initial method review: comment `7569821d-99fa-4030-bc4b-a0a87b373474`, created 2026-11-21T19:44:31.009Z; batch read-back passed with exact body and parsed YAML.

All returned records passed YAML parsing and finding-set parity. Initial comments match the examined scope in this artifact; disposition comments contain a nonempty fixed after_excerpt for every original finding. No duplicate marker was present in the authorized initial latest-100 comment batch.
