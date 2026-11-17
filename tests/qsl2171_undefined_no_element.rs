// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-411 (TC-959): `Undefined::NoElement`, the carrier of the FSpec undefined
//! reason `no-element`. The variant does not exist yet, so the stub reads the
//! declaration source; the implementation replaces it with typed comparisons.

use filament_exact::{Outcome, Undefined};
use ix_trace_rs::trace;

const OUTCOME_SOURCE: &str = include_str!("../src/outcome.rs");

#[trace("TC-959", "FR-411-AC-1", "FR-411-AC-2", "FR-411-AC-3")]
#[test]
#[ignore = "QSL-2171: Undefined::NoElement for an unmet index obligation is not implemented yet"]
fn tc_959_no_element_is_a_field_free_undefined_reason() {
    let declaration = OUTCOME_SOURCE
        .split("pub enum Undefined {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("the Undefined declaration");
    assert!(declaration.contains("    NoElement,"), "NoElement is a field-free variant of Undefined");
    assert!(declaration.contains("`no-element`"), "its doc comment names the FSpec reason");
    assert_eq!(declaration.matches("NoElement").count(), 1, "one index-read variant");
    for other in [
        Undefined::DivisionByZero,
        Undefined::IeeeNotFinite,
        Undefined::EmptyReduction,
        Undefined::NoneValue,
        Undefined::SumOutOfDomain,
    ] {
        assert!(Outcome::<i32>::Undefined(other).completed().is_none());
    }
}
