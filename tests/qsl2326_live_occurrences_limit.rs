// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-412 (TC-960): `LimitKind::LiveOccurrences`, named `live_occurrences`,
//! outside `LimitKind::ALL`. The variant does not exist yet, so the stub reads
//! the declaration source beside the typed checks that already compile; the
//! implementation replaces the source read with typed assertions.

use filament_exact::LimitKind;
use ix_trace_rs::trace;

const ACCOUNTING_SOURCE: &str = include_str!("../src/accounting.rs");

#[trace("TC-960", "FR-412-AC-1", "FR-412-AC-2")]
#[test]
#[ignore = "QSL-2326: LimitKind::LiveOccurrences (live_occurrences, outside ALL) is not implemented yet"]
fn tc_960_live_occurrences_is_named_and_outside_all() {
    assert!(
        ACCOUNTING_SOURCE.contains("Self::LiveOccurrences => \"live_occurrences\""),
        "as_str spells the variant live_occurrences"
    );
    assert_eq!(
        LimitKind::ALL.len(),
        10,
        "ALL stays the ScalarLimitsV1 counters"
    );
    assert!(LimitKind::ALL
        .iter()
        .all(|kind| kind.as_str() != "live_occurrences"));
    assert_eq!(LimitKind::from_code("live_occurrences"), None);
    for kind in LimitKind::ALL {
        assert_eq!(LimitKind::from_code(kind.as_str()), Some(kind));
    }
}
