---
id: SR-17940486709564847048060
title: gap-analysis review of QSL-2158 shared work owner
type: SpecReview
analysis: gap-analysis
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

The engine reports seven newly added criteria untagged; this prevents acceptance regardless of the passing runtime tests.

Frozen PR191 source reviewed independently against the approved owner contract. Ticket: QSL-2158. Head: d1bcce010f226c75c25b1ed171f02ee981e9ab03; base: b26a8e4c592503fccdc4a0e3d2cce23dd8487616. The reviewed source has eight changed files (412 additions, 19 deletions), no dependency, package version or CI changes. No applicable AssuranceProfile exists under spec/. No product source, specification, branch, tracker or GitHub comments were mutated by this reviewer.

Actual Titan execution at this exact head: cargo check --locked --tests -p filament-exact rc0; default accounting::work_scope_tests 9 passed; test-support same prefix 9 passed; ir673_core charge_point_vocabulary_is_unique_and_round_trips --exact 1 passed (19 total runtime passes, zero failures/ignored). These are the authorized focused receipts, not a broad gate, new Kani proof or downstream integration result. No local Cargo command ran.

Publication remains pending the user's Linear hold and root coordination. No Linear reads or writes occurred. The known has-review label is supplied by the dispatch, not independently queried. Draft YAML is locally checked; tracker dedup/read-back remains pending.

## Verdict

FAIL — The engine reports seven newly added criteria untagged; this prevents acceptance regardless of the passing runtime tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-359-AC-13 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-13")] to each test asserting it. | src/accounting/work_scope_tests.rs:24 |
| FND-002 | high | FR-359-AC-14 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-14")] to each test asserting it. | src/accounting/work_scope_tests.rs:55 |
| FND-003 | high | FR-359-AC-15 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-15")] to each test asserting it. | src/accounting/work_scope_tests.rs:247 |
| FND-004 | high | FR-359-AC-16 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-16")] to each test asserting it. | src/accounting/work_scope_tests.rs:125 |
| FND-005 | high | FR-359-AC-17 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-17")] to each test asserting it. | src/accounting/work_scope_tests.rs:151 |
| FND-006 | high | FR-359-AC-18 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-18")] to each test asserting it. | src/accounting/work_scope_tests.rs:218 |
| FND-007 | high | FR-359-AC-19 is untagged in the computed matrix: its real test uses documentation-only Trace text, not the active Rust trace attribute. Add #[trace("FR-359-AC-19")] to each test asserting it. | src/accounting/work_scope_tests.rs:205 |

## Coverage

Reconciliation: quoin matrix (quoin 0.36.8, quire 0.52.0 / engine 0.62.2), cross-checked against quire matrix and quire coverage. Full repository rollup: 472 criteria, 278 tagged, 134 untagged, 10 tagged-by-ignored-test, 50 method-without-symbol. All 472 have no run evidence in the local store; the dispatched Titan results remain separately attributed, not fabricated store evidence. Within the changed set, all seven new FR-359 criteria are untagged; FR-368-AC-1 and AC-3 are tagged to the exhaustive catalog test. Coverage census declares only rust-trace-attribute for Rust; the unbound comments are not missing runtime tests.

Plan completion: not assessed

Reverse-gap/stub inspection inventoried 75 Rust source files and public exports, checked new API/accounting/candidate behavior against FR-359/FR-368, and scanned source for todo!, unimplemented!, TODO and FIXME (zero hits). All changed behavior has an owning requirement, and all nine added tests contain behavioral assertions; zero changed untraced behaviors, source stubs or test stubs were found. Existing unrelated matrix gaps and ignored-only criteria are contextual corpus results, not newly investigated defects or a whole-repository completion claim. Detailed reverse-mapping of every pre-existing public item was not completed in this bounded PR review. Optional semantic gap review was not requested and was skipped; independent code/spec faithfulness checks belong to the code-review method.

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

## Dispositions

Round 1; independently inspected head f698357016bab5caecfbcecc01b4e56ff5ad56c0.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-002 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-003 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-004 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-005 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-006 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |
| FND-007 | fixed | Real Rust trace attributes now bind the relevant criterion; independently verified in the computed matrix at f698357016bab5caecfbcecc01b4e56ff5ad56c0. |

## Disposition evidence (round 1)

The actual source now imports ix_trace_rs::trace and applies nine real trace attributes. Independent quire matrix confirms FR-359-AC-13..19 tagged through nine binders (AC-15 and AC-18 have two each). All nine test bodies are unchanged after removing the new import/attributes, and production/spec/catalog files are byte-identical to the first-reviewed head. No new defect was found.

Titan at f698357016bab5caecfbcecc01b4e56ff5ad56c0: default accounting::work_scope_tests rc0/9 passed/0 failed/0 ignored; test-support same prefix rc0/9 passed/0 failed/0 ignored. Prior d1bcce010f226c75c25b1ed171f02ee981e9ab03 compile rc0 and catalog rc0/1 pass remain attributed to that earlier head; production/catalog/test bodies are unchanged. No local Cargo command ran.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-13")]

    #[test]

    fn separate_meters_share_original_bound_across_clone_and_drop() {'
- fnd: FND-002
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-14")]

    #[test]

    fn plan_reserves_shared_remainder_but_commits_one() {'
- fnd: FND-003
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-15")]

    #[test]

    fn local_size_and_later_result_denials_roll_back_shared_work() {


    #[trace("FR-359-AC-15")]

    #[test]

    fn equality_plan_result_denial_does_not_commit_reservation_or_size() {'
- fnd: FND-004
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-16")]

    #[test]

    fn exact_oversized_work_and_results_preserve_prefix_and_priority() {'
- fnd: FND-005
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-17")]

    #[test]

    fn cancellation_observer_can_read_scope_and_denials_do_not_spend_work() {'
- fnd: FND-006
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-18")]

    #[test]

    fn interval_capture_counts_only_this_meters_increments() {


    #[trace("FR-359-AC-18")]

    #[test]

    fn concurrent_meters_cannot_overdraw_one_entitlement() {'
- fnd: FND-007
  outcome: fixed
  after_excerpt: '#[trace("FR-359-AC-19")]

    #[test]

    fn unscoped_clones_remain_independent_and_value_equal() {'
```

## Disposition verdict (round 1)

PASS for the assigned PR change: all original findings in this method are fixed; zero new findings. The original review and verdict above remain historical and unmodified. Existing unrelated repository matrix gaps are not resolved by this disposition, which makes no whole-repository assurance claim.
