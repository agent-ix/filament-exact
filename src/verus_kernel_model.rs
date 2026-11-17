// SPDX-License-Identifier: AGPL-3.0-or-later
//! The kernel types the Verus proofs name without verifying, and the one
//! assumption made about the `Meter`. The file is compiled only by a Verus
//! run (`cfg(verus_keep_ghost)`).
//!
//! `Meter::charge_units` is not verified here. Its behaviour is the subject
//! of FR-359 and its Kani gate. The proofs assume only that an admitted
//! request is appended to the meter's admitted sequence and that a denied
//! request leaves that sequence unchanged.
#![allow(unsafe_code)]

use vstd::prelude::*;

use crate::accounting::{ChargePoint, Incomplete, LimitKind, Meter, UnitsCharge};
use crate::numeric::OrderingOperator;
use crate::outcome::{Outcome, Refusal, Undefined};

verus! {

#[verifier::external_type_specification]
pub struct ExChargePoint(ChargePoint);

#[verifier::external_type_specification]
pub struct ExLimitKind(LimitKind);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExMeter(Meter);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIncomplete(Incomplete);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExUndefined(Undefined);

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRefusal(Refusal);

#[verifier::external_type_specification]
#[verifier::reject_recursive_types(T)]
pub struct ExOutcome<T>(Outcome<T>);

/// The requests a meter has admitted, in order.
pub uninterp spec fn admitted(meter: Meter) -> Seq<UnitsCharge>;

pub assume_specification[ Meter::charge_units ](meter: &mut Meter, charge: UnitsCharge) -> (r: Result<
    (),
    Incomplete,
>)
    ensures
        r is Ok ==> admitted(*final(meter)) == admitted(*old(meter)).push(charge),
        r is Err ==> admitted(*final(meter)) == admitted(*old(meter)),
;

#[verifier::external_type_specification]
pub struct ExOrderingOperator(OrderingOperator);

/// Specification of `OrderingOperator::holds`.
pub open spec fn ordering_holds(operator: OrderingOperator, ordering: core::cmp::Ordering) -> bool {
    match operator {
        OrderingOperator::Less => ordering == core::cmp::Ordering::Less,
        OrderingOperator::LessOrEqual => ordering != core::cmp::Ordering::Greater,
        OrderingOperator::Greater => ordering == core::cmp::Ordering::Greater,
        OrderingOperator::GreaterOrEqual => ordering != core::cmp::Ordering::Less,
    }
}

} // verus!
