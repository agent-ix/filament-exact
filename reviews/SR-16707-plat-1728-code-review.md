---
id: SR-16707
title: code-review of PLAT-1728/PLAT-1729 identity README and feature-gated rustdoc links
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-exact@7fc5c678ea150ca83c887e10f70b7769d94a7193; README.md, src/accounting.rs, src/clock_reference.rs; Ticket PLAT-1728, PLAT-1729"
review_set: subset
---

## Summary

Ticket: PLAT-1728 (also PLAT-1729). Review of PR #196 at `7fc5c678ea150ca83c887e10f70b7769d94a7193`, scope `git diff origin/main...HEAD` (one commit, three files, doc text only). The README no longer mentions `from_digest`; the four constructors it names exist with matching names (`NodeKey::new` src/node.rs:806, `OriginalDeclarationKey::new` src/identity.rs:25, `EffectiveId::new` src/identity.rs:61, `UnitId::declared` src/identity.rs:211), none feature-gated, all re-exported from the crate root. The two doc-comment links changed to plain code spans point at items gated by `test-support` (`Meter::admitted_charges`) and `json-object` (`ClockPlaceReference::new`); the edits are inside `///` comments only, so behavior is unchanged. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked -j 2` passes on the head with default features. A grep of intra-doc links to every `cfg(feature = ...)` item in src/ found no remaining link to a gated item.

## Verdict

PASS with three low findings. Rust review ran as the language lane of this method; no Rust idiom, safety, or API issue (doc comments only).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Identity table row still calls the identities "digest identities", contradicting the rewritten paragraph | README.md:91 |
| FND-002 | low | "checked components" overstates three of the four named constructors, which perform no check | README.md:96-99 |
| FND-003 | low | Rewritten paragraph breaks the file's line wrap (one line ~88 columns) | README.md:99-100 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified in 725fe03d: README.md:91 now reads "the other structural identities". |
| FND-002 | fixed | Verified in 725fe03d: README.md:96-99 says only `NodeKey::new` validates; the other three carry caller-admitted components. |
| FND-003 | fixed | Verified in 725fe03d: the rewrapped paragraph has no line over 80 columns. |
