// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! FR-412 (TC-960): the evaluation session limit is named by the kernel but
//! stays outside scalar accounting; a session can carry it in `Incomplete`.

use filament_exact::{
    Charge, ChargePoint, Incomplete, Integer, LimitKind, Meter, Outcome, ScalarLimits,
};
use ix_trace_rs::trace;

#[trace("TC-960", "FR-412-AC-1", "FR-412-AC-2")]
#[test]
fn tc_960_live_occurrences_is_named_and_outside_all() {
    assert_eq!(LimitKind::LiveOccurrences.as_str(), "live_occurrences");
    assert_ne!(LimitKind::LiveOccurrences, LimitKind::ValueOccurrences);
    assert!(LimitKind::LiveOccurrences > LimitKind::ResultUnits);
    assert!(!LimitKind::LiveOccurrences.is_cumulative());
    assert_eq!(LimitKind::ALL.len(), 10);
    assert!(!LimitKind::ALL.contains(&LimitKind::LiveOccurrences));
    assert_eq!(LimitKind::from_code("live_occurrences"), None);
    for kind in LimitKind::ALL {
        assert_eq!(LimitKind::from_code(kind.as_str()), Some(kind));
    }
}

#[trace("TC-960", "FR-412-AC-3")]
#[test]
fn tc_960_live_occurrences_size_requests_are_ignored_by_scalar_meter() {
    let charge = Charge::new(ChargePoint::FunctionCall);
    assert_eq!(
        charge.clone().size(LimitKind::LiveOccurrences, 5),
        charge.clone()
    );
    assert_eq!(
        charge.clone().exact_size(
            LimitKind::LiveOccurrences,
            Integer::from(u64::MAX).add(&Integer::one())
        ),
        charge
    );

    let mut meter = Meter::new(ScalarLimits {
        integer_bits: u64::MAX,
        decimal_digits: u64::MAX,
        scale_expansion: u64::MAX,
        text_input_bytes: u64::MAX,
        text_scalars: u64::MAX,
        normalized_scalars: u64::MAX,
        unit_edges: u64::MAX,
        value_occurrences: u64::MAX,
        work_units: u64::MAX,
        result_units: u64::MAX,
    });
    meter.enable_charge_accounting();
    meter
        .charge(Charge::new(ChargePoint::FunctionCall).size(LimitKind::LiveOccurrences, 5))
        .unwrap();
    assert_eq!(meter.consumed(LimitKind::LiveOccurrences), 0);
    assert_eq!(
        meter
            .take_charge_accounting()
            .unwrap()
            .amount(LimitKind::LiveOccurrences),
        0
    );
}

#[trace("TC-960", "FR-412-AC-4")]
#[test]
fn tc_960_external_session_can_carry_live_occurrences_incomplete() {
    let record = Incomplete {
        limit_kind: LimitKind::LiveOccurrences,
        limit: 1_048_576,
        consumed: 1_048_576,
        next_charge: Integer::from(2_u64),
        charge_point: ChargePoint::FunctionCall,
    };
    let Outcome::<()>::Incomplete(read_back) = Outcome::Incomplete(record) else {
        panic!("the typed session limit remains an incomplete outcome");
    };
    assert_eq!(read_back.limit_kind, LimitKind::LiveOccurrences);
    assert_eq!(read_back.limit, 1_048_576);
    assert_eq!(read_back.consumed, 1_048_576);
    assert_eq!(read_back.next_charge, Integer::from(2_u64));
    assert_eq!(read_back.charge_point, ChargePoint::FunctionCall);
}
