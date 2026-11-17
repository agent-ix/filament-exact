// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-411 (TC-959): the kernel carries one field-free undefined reason for an
//! unmet collection-index obligation. Source-level limits are checked in review.

use filament_exact::{Outcome, Undefined};
use ix_trace_rs::trace;

const OUTCOME_SOURCE: &str = include_str!("../src/outcome.rs");

#[trace("TC-959", "FR-411-AC-1", "FR-411-AC-2", "FR-411-AC-3")]
#[test]
fn tc_959_no_element_is_a_field_free_undefined_reason() {
    let reason = Undefined::NoElement;
    for other in [
        Undefined::DivisionByZero,
        Undefined::IeeeNotFinite,
        Undefined::EmptyReduction,
        Undefined::NoneValue,
        Undefined::SumOutOfDomain,
    ] {
        assert_ne!(reason, other);
    }
    assert_eq!(reason, reason.clone());
    assert!(Outcome::<i32>::Undefined(reason).completed().is_none());

    let declaration = OUTCOME_SOURCE
        .split("pub enum Undefined {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("the Undefined declaration");
    assert!(declaration.contains("    NoElement,"));
    assert!(declaration.contains("FSL"));
    assert!(declaration.contains("direct kernel evaluation of an unlinked expression"));
    assert_eq!(declaration.matches("NoElement").count(), 1);
}
