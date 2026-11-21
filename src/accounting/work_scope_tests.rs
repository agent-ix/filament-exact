// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared-work tests using the real meter transitions.
use ix_trace_rs::trace;

use super::*;

fn limits(work_units: u64) -> ScalarLimits {
    ScalarLimits {
        integer_bits: 8,
        decimal_digits: 8,
        scale_expansion: 8,
        text_input_bytes: 8,
        text_scalars: 8,
        normalized_scalars: 8,
        unit_edges: 8,
        value_occurrences: 8,
        work_units,
        result_units: 1,
    }
}

fn candidate(work: u64) -> Charge {
    Charge::new(ChargePoint::ModelCandidate).work(Integer::from(work))
}

/// Trace: FR-359-AC-13
#[trace("FR-359-AC-13")]
#[test]
fn separate_meters_share_original_bound_across_clone_and_drop() {
    let scope = WorkScope::new(limits(5));
    assert_eq!(scope, scope.clone());
    assert_ne!(scope, WorkScope::new(limits(5)));
    let mut first = scope.meter();
    first.charge(candidate(3)).unwrap();
    let mut second = scope.clone().meter();
    let refusal = second.charge(candidate(3)).unwrap_err();
    assert_eq!(
        refusal,
        Incomplete {
            limit_kind: LimitKind::WorkUnits,
            limit: 5,
            consumed: 3,
            next_charge: Integer::from(3_u64),
            charge_point: ChargePoint::ModelCandidate,
        }
    );
    let mut cloned = first.clone();
    drop(first);
    cloned.charge(candidate(2)).unwrap();
    drop(cloned);
    assert_eq!(scope.meter().charge(candidate(1)).unwrap_err().consumed, 5);
    assert_eq!(second.consumed(LimitKind::WorkUnits), 5);
    assert_eq!(scope.consumed_work(), 5);
    assert_eq!(scope, scope.clone());
    assert_ne!(scope, WorkScope::new(limits(5)));
}

/// Trace: FR-359-AC-14
#[trace("FR-359-AC-14")]
#[test]
fn plan_reserves_shared_remainder_but_commits_one() {
    for native in [false, true] {
        let scope = WorkScope::new(limits(5));
        scope.meter().charge(candidate(2)).unwrap();
        let mut plan = scope.meter();
        plan.enable_charge_accounting();
        let run = |meter: &mut Meter| {
            if native {
                meter.charge_plan_one_pair()
            } else {
                meter.charge_plan(&Integer::one())
            }
        };
        run(&mut plan).unwrap();
        assert_eq!(scope.consumed_work(), 3);
        assert_eq!(
            plan.take_charge_accounting()
                .unwrap()
                .amount(LimitKind::WorkUnits),
            1
        );
        let refusal = run(&mut plan).unwrap_err();
        assert_eq!(refusal.limit, 5);
        assert_eq!(refusal.consumed, 3);
        assert_eq!(refusal.next_charge, Integer::from(3_u64));
        assert_eq!(refusal.charge_point, ChargePoint::EqualityPlan);
        assert_eq!(scope.consumed_work(), 3);
    }
}

/// Trace: FR-359-AC-15
#[trace("FR-359-AC-15")]
#[test]
fn local_size_and_later_result_denials_roll_back_shared_work() {
    for general in [false, true] {
        let scope = WorkScope::new(limits(8));
        let mut meter = scope.meter();
        meter
            .charge(candidate(2).size(LimitKind::IntegerBits, 4).results(1))
            .unwrap();
        let request = |bits| {
            let charge = candidate(2).size(LimitKind::IntegerBits, bits).results(1);
            if general {
                charge
                    .size(LimitKind::TextScalars, 2)
                    .size(LimitKind::UnitEdges, 2)
            } else {
                charge
            }
        };
        assert_eq!(
            meter.charge(request(9)).unwrap_err().limit_kind,
            LimitKind::IntegerBits
        );
        assert_eq!(
            meter.charge(request(8)).unwrap_err().limit_kind,
            LimitKind::ResultUnits
        );
        assert_eq!(scope.consumed_work(), 2);
        assert_eq!(meter.consumed(LimitKind::IntegerBits), 4);
        assert_eq!(meter.admission_count(), 1);
        let mut fresh = scope.meter();
        assert_eq!(fresh.consumed(LimitKind::IntegerBits), 0);
        assert_eq!(fresh.consumed(LimitKind::ResultUnits), 0);
        fresh.charge(candidate(2).results(1)).unwrap();
        assert_eq!(scope.consumed_work(), 4);
    }
}

/// Trace: FR-359-AC-16
#[trace("FR-359-AC-16")]
#[test]
fn exact_oversized_work_and_results_preserve_prefix_and_priority() {
    let scope = WorkScope::new(limits(u64::MAX));
    scope.meter().charge(candidate(u64::MAX - 1)).unwrap();
    let huge = Integer::from(u64::MAX).add(&Integer::one());
    let mut meter = scope.meter();
    let refusal = meter
        .charge(candidate(0).work(huge.clone()).exact_results(huge.clone()))
        .unwrap_err();
    assert_eq!(refusal.limit_kind, LimitKind::WorkUnits);
    assert_eq!(refusal.limit, u64::MAX);
    assert_eq!(refusal.consumed, u64::MAX - 1);
    assert_eq!(refusal.next_charge, huge);
    assert_eq!(
        meter
            .charge(candidate(1).exact_results(huge.clone()))
            .unwrap_err()
            .limit_kind,
        LimitKind::ResultUnits
    );
    assert_eq!(scope.consumed_work(), u64::MAX - 1);
    meter.charge(candidate(1)).unwrap();
    assert_eq!(meter.charge(candidate(1)).unwrap_err().consumed, u64::MAX);
}

/// Trace: FR-359-AC-17
#[trace("FR-359-AC-17")]
#[test]
fn cancellation_observer_can_read_scope_and_denials_do_not_spend_work() {
    let scope = WorkScope::new(limits(6));
    scope.meter().charge(candidate(2)).unwrap();
    let observed = scope.clone();
    let cancel = Cancel::observing(move || {
        assert_eq!(observed.consumed_work(), 2);
    });
    cancel.cancel(crate::CancelCause::Deadline);
    let mut meter = scope
        .meter()
        .with_cancel(cancel.clone())
        .with_injected_denial(InjectedDenial {
            point: ChargePoint::ModelCandidate,
            occurrence: NonZeroU64::new(1).unwrap(),
        });
    let cancelled = meter.charge(candidate(3)).unwrap_err();
    assert_eq!((cancelled.limit, cancelled.consumed), (2, 2));
    assert_eq!(cancelled.next_charge, Integer::from(3_u64));
    assert_eq!(cancel.tripped(), Some(crate::CancelCause::Deadline));
    let live = Cancel::new();
    let mut meter = meter.with_cancel(live.clone());
    let injected = meter.charge(candidate(3)).unwrap_err();
    assert_eq!((injected.limit, injected.consumed), (2, 2));
    assert_eq!(live.tripped(), None);
    assert_eq!(scope.consumed_work(), 2);
    meter.charge(candidate(3)).unwrap();
    assert_eq!(scope.consumed_work(), 5);
}

/// Trace: FR-359-AC-18
#[trace("FR-359-AC-18")]
#[test]
fn interval_capture_counts_only_this_meters_increments() {
    let scope = WorkScope::new(limits(10));
    let mut meter = scope.meter();
    meter.enable_charge_accounting();
    meter.enable_charge_capture(2, 0);
    meter.charge(candidate(2)).unwrap();
    scope.meter().charge(candidate(4)).unwrap();
    meter.charge(candidate(3)).unwrap();
    let amounts = meter.take_charge_accounting().unwrap();
    assert_eq!(amounts.amount(LimitKind::WorkUnits), 5);
    assert_eq!(scope.consumed_work(), 9);
    let capture = meter.take_charge_capture().unwrap().unwrap();
    assert_eq!(capture.events().len(), 2);
    assert_eq!(capture.events()[0].work_units, Integer::from(2_u64));
    assert_eq!(capture.events()[1].work_units, Integer::from(3_u64));
    assert!(capture
        .events()
        .iter()
        .all(|event| event.admission == ChargeAdmission::Admitted));
}

/// Trace: FR-359-AC-19
#[trace("FR-359-AC-19")]
#[test]
fn unscoped_clones_remain_independent_and_value_equal() {
    let mut meter = Meter::new(limits(3));
    meter.charge(candidate(2)).unwrap();
    let mut cloned = meter.clone();
    assert_eq!(cloned, meter);
    cloned.charge(candidate(1)).unwrap();
    assert_eq!(meter.consumed(LimitKind::WorkUnits), 2);
    assert_eq!(cloned.consumed(LimitKind::WorkUnits), 3);
    assert_eq!(Meter::new(limits(3)).consumed(LimitKind::WorkUnits), 0);
}

/// Trace: FR-359-AC-18
#[trace("FR-359-AC-18")]
#[test]
fn concurrent_meters_cannot_overdraw_one_entitlement() {
    fn assert_sync<T: Send + Sync>() {}
    assert_sync::<WorkScope>();
    assert_sync::<Meter>();
    let scope = WorkScope::new(limits(3));
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let scope = scope.clone();
        let barrier = barrier.clone();
        workers.push(std::thread::spawn(move || {
            let mut meter = scope.meter();
            barrier.wait();
            meter.charge(candidate(2))
        }));
    }
    let outcomes: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    let refusal = outcomes.into_iter().find_map(Result::err).unwrap();
    assert_eq!((refusal.limit, refusal.consumed), (3, 2));
    assert_eq!(refusal.next_charge, Integer::from(2_u64));
    assert_eq!(scope.consumed_work(), 2);
}

/// Trace: FR-359-AC-15
#[trace("FR-359-AC-15")]
#[test]
fn equality_plan_result_denial_does_not_commit_reservation_or_size() {
    for native in [false, true] {
        let scope = WorkScope::new(ScalarLimits {
            result_units: 0,
            ..limits(6)
        });
        scope.meter().charge(candidate(2)).unwrap();
        let mut meter = scope.meter();
        let result = if native {
            meter.charge_plan_one_pair()
        } else {
            meter.charge_plan(&Integer::one())
        };
        let refusal = result.unwrap_err();
        assert_eq!(refusal.limit_kind, LimitKind::ResultUnits);
        assert_eq!((refusal.limit, refusal.consumed), (0, 0));
        assert_eq!(scope.consumed_work(), 2);
        assert_eq!(meter.consumed(LimitKind::ValueOccurrences), 0);
        assert_eq!(meter.admission_count(), 0);
    }
}
