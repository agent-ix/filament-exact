// SPDX-License-Identifier: AGPL-3.0-or-later
//! Occurrence-pair equality planning.
//!
//! The module holds the operand-agnostic occurrence-pair plan: [`plan_pairs`],
//! [`plan_equality`], `planned_equality`, [`EqualityPlan`] and
//! [`EqualityEvaluation`]. Anything that
//! needs a declaration registry or a unit graph is the caller's, not the
//! kernel's: selecting and running the top-level text/enum/quantity schedules
//! ahead of the generic occurrence-pair plan is done above the kernel with the
//! registry and unit graph the caller holds. An enum's ordering and case lookup
//! needs no declaration here, since `ValueType::Enum` carries its inline
//! `EnumShape` set (see `crate::value`'s module doc comment).
//!
//! The leaf comparison inside `plan_pairs` works on the kernel's bare
//! `Value::Enum` payload: two enum values compare equal exactly when their
//! `VariantId` digests are equal. A checked program guarantees both operands
//! share one declared `ValueType::Enum(EnumShape)` before this ever runs -- the
//! same invariant every other leaf type here relies on (an `Integer` carries no
//! declared bound either) -- so this needs no same-enum check of its own. A
//! quantity pair compares equal only in the same unit.
//!
//! Two composites are compared by declaration key first. A caller may hold a
//! value whose most-specific declaration is a descendant of the declaration a
//! slot names, so two operands of one static type can carry different
//! declarations. That is ordinary, not a malformed program: the pair is
//! unequal, with no descent into either value's slots. The kernel carries no
//! declaration graph and treats no declaration as the ancestor of another. A
//! composite against a non-composite, and two composites of one declaration
//! with different slot counts, still refuse with the checked invariant.

use crate::accounting::{Charge, ChargePoint, Incomplete, LimitKind, Meter};
use crate::integer::Integer;
use crate::key::compare_membership_keys;
use crate::outcome::{CheckedInvariantCause, Outcome, Refusal, Stop};
use crate::value::{FieldValue, Value};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// The complete occurrence-pair plan of one planned equality.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EqualityPlan {
    pair_events: Integer,
}

impl EqualityPlan {
    /// Wrap an already-computed pair count. `pub`, not merely a private
    /// struct literal: `EqualityPlan` is a plain data holder with no
    /// invariant beyond "this many pairs were planned", so it carries no
    /// accounting charge of its own; a caller's own occurrence-pair walk (over its
    /// own `Value`, a distinct type from this crate's) needs this to build
    /// one from a count it computed itself, the same way this module's own
    /// [`plan_equality`] does internally.
    pub fn new(pair_events: Integer) -> Self {
        Self { pair_events }
    }

    /// The exact number of planned `equality.pair` events.
    pub fn pair_events(&self) -> &Integer {
        &self.pair_events
    }
}

/// The result and pair-event count produced by one completed metered equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EqualityEvaluation {
    equal: bool,
    plan: EqualityPlan,
}

impl EqualityEvaluation {
    /// The exact equality relation from the formed plan.
    pub fn equal(&self) -> bool {
        self.equal
    }

    /// The exact pair-event plan used by this evaluation.
    pub fn plan(&self) -> &EqualityPlan {
        &self.plan
    }
}

/// Form the plan of two completed operands of one type, without charge. A
/// reference pair of different universes refuses with `foreign_reference`.
///
/// **Stays `pub` and unmetered.** This
/// walks the whole occurrence-pair tree of caller-supplied values with no
/// meter, which is the risk profile of any unmetered walk. The difference
/// here is that "without charge" is this function's entire documented purpose: it lets a caller size an equality check's cost
/// (via [`EqualityPlan::pair_events`]) *before* spending [`planned_equality`]'s
/// metered budget on it. Giving it a `&mut Meter` would defeat that purpose,
/// and demoting it to `pub(crate)` deletes a capability callers rely on that has
/// no in-crate substitute (`planned_equality` needs the `equal` bool this
/// plan discards, so it cannot be rewritten to call this instead). Left
/// `pub` and documented: a caller that does not want unbounded work must
/// bound its own operand size before calling this, the same way any
/// pre-metering admission check must.
///
/// **Round 2 correction:** "size the cost before spending the budget"
/// overstates what this buys. The walk here costs exactly what
/// [`planned_equality`]'s does over the same pair -- it is not a cheaper
/// estimate. Calling this first does not let a caller *avoid* paying that
/// cost; it lets them pay it once unmetered to decide whether to pay it
/// again metered.
pub fn plan_equality(left: &Value, right: &Value) -> Result<EqualityPlan, Refusal> {
    plan_pairs(left, right).map(|plan| EqualityPlan {
        pair_events: plan.pairs,
    })
}

/// The equality schedule over completed operands of one type: charges
/// `equality.plan-form`, `equality.plan`, one `equality.pair` per planned
/// occurrence-path pair and `equality.result-retain`.
pub fn planned_equality(left: &Value, right: &Value, meter: &mut Meter) -> Outcome<bool> {
    match planned_equality_with_plan(left, right, meter) {
        Outcome::Completed(result) => Outcome::Completed(result.equal()),
        Outcome::Undefined(reason) => Outcome::Undefined(reason),
        Outcome::Refused(reason) => Outcome::Refused(reason),
        Outcome::Incomplete(record) => Outcome::Incomplete(record),
    }
}

/// Evaluate equality and return the relation and pair count from the same plan.
/// The returned record exists only after every scheduled meter charge succeeds.
pub fn planned_equality_with_plan(
    left: &Value,
    right: &Value,
    meter: &mut Meter,
) -> Outcome<EqualityEvaluation> {
    Outcome::from_stop(planned_equality_stop(left, right, meter))
}

fn planned_equality_stop(
    left: &Value,
    right: &Value,
    meter: &mut Meter,
) -> Result<EqualityEvaluation, Stop> {
    charge_plan_form(&left.occ(), &right.occ(), meter)?;
    let plan = plan_pairs(left, right).map_err(Stop::Refused)?;
    charge_pairs(&plan.pairs, meter)?;
    Ok(EqualityEvaluation {
        equal: plan.equal,
        plan: EqualityPlan {
            pair_events: plan.pairs,
        },
    })
}

/// `equality.plan-form`: the larger occurrence count as size and the sum of
/// both counts as work. The native equality entries charge the same amounts in `u64`
/// units (`charge_equality_schedule`), pinned equal by their differential tests.
pub(crate) fn charge_plan_form(
    left_occ: &Integer,
    right_occ: &Integer,
    meter: &mut Meter,
) -> Result<(), Incomplete> {
    meter.charge(
        Charge::new(ChargePoint::EqualityPlanForm)
            .exact_size(
                LimitKind::ValueOccurrences,
                left_occ.clone().max(right_occ.clone()),
            )
            .work(left_occ.add(right_occ)),
    )
}

/// `equality.plan`, one `equality.pair` per planned pair and
/// `equality.result-retain`.
pub(crate) fn charge_pairs(pairs: &Integer, meter: &mut Meter) -> Result<(), Incomplete> {
    meter.charge_plan(pairs)?;
    let mut remaining = pairs.clone();
    while !remaining.is_zero() {
        meter.charge(Charge::new(ChargePoint::EqualityPair))?;
        remaining = remaining.sub(&Integer::one());
    }
    meter.charge(Charge::new(ChargePoint::EqualityResultRetain).results(1))
}

/// A formed plan: its pair count and the relation's Boolean.
pub(crate) struct PlannedPairs {
    pub(crate) pairs: Integer,
    pub(crate) equal: bool,
}

/// One node of the occurrence-pair tree still to be formed.
#[derive(Clone, Copy)]
enum Pair<'a> {
    MapKeys(&'a Value, &'a Value, LeafRelation),
    Values(&'a Value, &'a Value, LeafRelation),
    Slots(&'a FieldValue, &'a FieldValue, LeafRelation),
}

type MapEntryMatch<'a> = (&'a Value, &'a Value, &'a Value, &'a Value);

/// Walk the occurrence-pair tree of two values of one static type. Two
/// composites of different declarations are one pair and unequal: a caller
/// may hold a conforming descendant where a slot names its ancestor, and the
/// kernel decides that pair from the two declaration keys alone (FR-376).
/// Operands of unlike variants, two composites of one declaration with
/// different slot counts, two populations and two collections of different
/// kinds are malformed, and refuse with the checked invariant.
pub(crate) fn plan_pairs(left: &Value, right: &Value) -> Result<PlannedPairs, Refusal> {
    plan_pairs_with_relation(left, right, LeafRelation::Ordinary)
}

/// Collection membership uses bit identity at float leaves, including nested ones.
pub(crate) fn plan_membership_pairs(left: &Value, right: &Value) -> Result<PlannedPairs, Refusal> {
    plan_pairs_with_relation(left, right, LeafRelation::Membership)
}

#[derive(Clone, Copy)]
enum LeafRelation {
    Ordinary,
    Membership,
}

fn plan_pairs_with_relation(
    left: &Value,
    right: &Value,
    relation: LeafRelation,
) -> Result<PlannedPairs, Refusal> {
    let mut pairs = Integer::zero();
    let mut equal = true;
    let mut pending = vec![Pair::Values(left, right, relation)];
    while let Some(pair) = pending.pop() {
        let (left, right, relation) = match pair {
            Pair::MapKeys(left, right, relation) => {
                let key_plan = match relation {
                    LeafRelation::Ordinary => plan_pairs(left, right)?,
                    LeafRelation::Membership => plan_membership_pairs(left, right)?,
                };
                pairs = pairs.add(&key_plan.pairs);
                equal &= key_plan.equal;
                continue;
            }
            Pair::Slots(FieldValue::Present(left), FieldValue::Present(right), relation) => {
                (left, right, relation)
            }
            Pair::Slots(FieldValue::Absent, FieldValue::Absent, _)
            | Pair::Slots(FieldValue::Null, FieldValue::Null, _) => continue,
            Pair::Slots(FieldValue::Present(_) | FieldValue::Absent | FieldValue::Null, _, _) => {
                equal = false;
                continue;
            }
            Pair::Values(left, right, relation) => (left, right, relation),
        };
        pairs = pairs.add(&Integer::one());
        pairs = pairs.add(&Integer::one());
        let leaf = match (left, right) {
            (Value::Float(l), Value::Float(r)) if l.width() == r.width() => match relation {
                LeafRelation::Ordinary => l.numeric_equal(*r),
                LeafRelation::Membership => l.bits() == r.bits(),
            },
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Integer(l), Value::Integer(r)) => l == r,
            (Value::Rational(l), Value::Rational(r)) => l == r,
            (Value::Decimal(l), Value::Decimal(r)) => l.numerically_equal(r),
            (Value::Quantity(l), Value::Quantity(r)) if l.unit() == r.unit() => {
                l.magnitude() == r.magnitude()
            }
            (Value::Text(l), Value::Text(r))
                if l.text_type().profile() == r.text_type().profile() =>
            {
                l.retained() == r.retained()
            }
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Uuid(l), Value::Uuid(r)) => l == r,
            (Value::Timestamp(l), Value::Timestamp(r)) => l == r,
            // Identity and equality use the `VariantId` only
            // -- the paired rank is ignored here, exactly as it is
            // absent from `ValueType::Enum`'s own admission-checked identity.
            (Value::Enum(l), Value::Enum(r)) => l.variant() == r.variant(),
            (Value::Reference(l), Value::Reference(r)) => {
                if l.universe() != r.universe() {
                    return Err(Refusal::ForeignReference {
                        required: l.universe().clone(),
                        supplied: r.universe().clone(),
                    });
                }
                l == r
            }
            (Value::Option(l), Value::Option(r)) => match (l.payload(), r.payload()) {
                (Some(l), Some(r)) => {
                    pending.push(Pair::Values(l, r, relation));
                    continue;
                }
                (l, r) => l.is_some() == r.is_some(),
            },
            // Composites of different declarations are unequal; the kernel
            // holds no declaration graph, so the keys alone decide and no
            // slot of either value is visited (FR-376).
            (Value::Composite(l), Value::Composite(r)) if l.declaration() != r.declaration() => {
                false
            }
            (Value::Composite(l), Value::Composite(r)) if l.slots().len() == r.slots().len() => {
                let slots = l.slots().iter().zip(r.slots()).rev();
                pending.extend(slots.map(|(l, r)| Pair::Slots(l, r, relation)));
                continue;
            }
            (Value::Union(l), Value::Union(r)) if l.declaration() == r.declaration() => {
                if l.variant() != r.variant() {
                    false
                } else if l.payload().len() != r.payload().len() {
                    return Err(Refusal::CheckedInvariant {
                        cause: CheckedInvariantCause::ValueKindMismatch,
                    });
                } else {
                    pending.extend(
                        l.payload()
                            .iter()
                            .zip(r.payload())
                            .rev()
                            .map(|(l, r)| Pair::Values(l, r, relation)),
                    );
                    continue;
                }
            }
            (Value::Collection(l), Value::Collection(r)) => {
                let kind = l.collection_type().kind();
                if kind != r.collection_type().kind() {
                    return Err(Refusal::CheckedInvariant {
                        cause: CheckedInvariantCause::CollectionKindMismatch,
                    });
                }
                let (l, r) = (l.elements(), r.elements());
                if l.len() != r.len() {
                    false
                } else {
                    let element_relation = if kind == crate::collection::CollectionKind::Sequence {
                        relation
                    } else {
                        LeafRelation::Membership
                    };
                    let ranks = l.iter().zip(r).rev();
                    pending.extend(ranks.map(|(l, r)| Pair::Values(l, r, element_relation)));
                    continue;
                }
            }
            (Value::Map(l), Value::Map(r)) => {
                if l.map_type() != r.map_type() || l.entries().len() != r.entries().len() {
                    false
                } else {
                    let mut order_keys = compare_map_membership_keys;
                    let mut equal_keys = match relation {
                        LeafRelation::Ordinary => compare_map_keys,
                        LeafRelation::Membership => compare_map_membership_keys,
                    };
                    let matches = map_entry_matches_by(
                        l.entries(),
                        r.entries(),
                        &mut order_keys,
                        &mut equal_keys,
                    )
                    .map_err(|()| Refusal::CheckedInvariant {
                        cause: CheckedInvariantCause::ValueKindMismatch,
                    })?;
                    if let Some(matches) = matches {
                        pending.extend(matches.into_iter().rev().flat_map(
                            |(left_key, left_value, right_key, right_value)| {
                                [
                                    Pair::MapKeys(left_key, right_key, relation),
                                    Pair::Values(left_value, right_value, relation),
                                ]
                            },
                        ));
                        continue;
                    } else {
                        false
                    }
                }
            }
            (Value::Population(_), Value::Population(_)) => {
                return Err(Refusal::CheckedInvariant {
                    cause: CheckedInvariantCause::PopulationPair,
                });
            }
            (
                Value::Boolean(_)
                | Value::Integer(_)
                | Value::Rational(_)
                | Value::Decimal(_)
                | Value::Float(_)
                | Value::Quantity(_)
                | Value::Text(_)
                | Value::String(_)
                | Value::Uuid(_)
                | Value::Timestamp(_)
                | Value::Enum(_)
                | Value::Population(_)
                | Value::Reference(_)
                | Value::Option(_)
                | Value::Composite(_)
                | Value::Union(_)
                | Value::Collection(_)
                | Value::Map(_),
                _,
            ) => {
                return Err(Refusal::CheckedInvariant {
                    cause: CheckedInvariantCause::ValueKindMismatch,
                });
            }
        };
        equal = equal && leaf;
    }
    Ok(PlannedPairs { pairs, equal })
}

/// Match two entry lists after sorting their borrowed views by key, retaining
/// the source maps' entry order. `Err(())` means a key pair lacks a canonical
/// typed order; `Ok(None)` means the maps have different keys.
fn map_entry_matches_by<'a>(
    left: &'a [(Value, Value)],
    right: &'a [(Value, Value)],
    order_keys: &mut impl FnMut(&Value, &Value) -> Option<Ordering>,
    equal_keys: &mut impl FnMut(&Value, &Value) -> Option<Ordering>,
) -> Result<Option<Vec<MapEntryMatch<'a>>>, ()> {
    if left.len() != right.len() {
        return Ok(None);
    }
    let left = ordered_map_entries_by(left, order_keys)?;
    let right = ordered_map_entries_by(right, order_keys)?;
    let mut matches = Vec::with_capacity(left.len());
    for (left_entry, right_entry) in left.into_iter().zip(right) {
        match equal_keys(&left_entry.0, &right_entry.0).ok_or(())? {
            Ordering::Equal => {
                matches.push((&left_entry.0, &left_entry.1, &right_entry.0, &right_entry.1))
            }
            Ordering::Less | Ordering::Greater => return Ok(None),
        }
    }
    Ok(Some(matches))
}

/// Return a borrowed view in ascending canonical key order.
fn ordered_map_entries_by<'a>(
    entries: &'a [(Value, Value)],
    compare: &mut impl FnMut(&Value, &Value) -> Option<Ordering>,
) -> Result<Vec<&'a (Value, Value)>, ()> {
    let mut ordered: Vec<_> = entries.iter().collect();
    let mut incomparable = false;
    ordered.sort_unstable_by(|left, right| match compare(&left.0, &right.0) {
        Some(ordering) => ordering,
        None => {
            incomparable = true;
            Ordering::Equal
        }
    });
    if incomparable {
        Err(())
    } else {
        Ok(ordered)
    }
}

/// Compare map keys for ordinary equality. Float's equality collapses signed
/// zero, matching `numericEqual`; NaNs retain their total-order position so
/// they can be aligned and then correctly fail ordinary equality.
fn compare_map_keys(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Float(left), Value::Float(right)) if left.width() == right.width() => {
            if left.numeric_equal(*right) {
                Some(Ordering::Equal)
            } else {
                Some(left.total_order_key().cmp(&right.total_order_key()))
            }
        }
        _ => compare_membership_keys(left, right),
    }
}

/// Compare map keys by membership identity, keeping signed zeros and NaN bit
/// patterns distinct. This provides a stable sort order for equality matching.
fn compare_map_membership_keys(left: &Value, right: &Value) -> Option<Ordering> {
    compare_membership_keys(left, right)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    #[cfg(feature = "test-support")]
    use alloc::vec::Vec;
    use core::num::NonZeroU64;
    use ix_trace_rs::trace;

    use super::*;
    use crate::accounting::{InjectedDenial, ScalarLimits};
    use crate::collection::{
        form, from_admitted, from_admitted_map, CardinalityBound, CollectionKind, CollectionType,
        MapType,
    };
    use crate::key::compare_keys;
    use crate::value::ValueType;

    /// Map equality is independent of supplied entry order, and ValueType
    /// admits a map value only under its declared map type.
    #[trace("TC-958", "FR-410-AC-1", "FR-410-AC-5")]
    #[test]
    fn map_equality_matches_entries_by_key_and_checks_map_type_admission() {
        let map_type = MapType::new(ValueType::String, ValueType::Integer, None).unwrap();
        let make_map = |entries: &[(&str, i64)]| {
            from_admitted_map(
                map_type.clone(),
                entries
                    .iter()
                    .map(|(key, value)| {
                        (
                            Value::String((*key).to_owned()),
                            Value::Integer((*value).into()),
                        )
                    })
                    .collect(),
            )
            .expect("unique map keys")
        };
        let first = make_map(&[("b", 2), ("a", 1)]);
        let empty = from_admitted_map(map_type.clone(), Vec::new()).expect("empty map");
        let reordered = make_map(&[("a", 1), ("b", 2)]);
        let changed = make_map(&[("a", 1), ("b", 3)]);
        let changed_keys = make_map(&[("a", 1), ("c", 2)]);
        let changed_count = make_map(&[("a", 1)]);

        assert_eq!(
            plan_equality(&first, &reordered).unwrap().pair_events(),
            &Integer::from(10_i64)
        );
        assert!(plan_pairs(&first, &reordered).unwrap().equal);
        assert!(!plan_pairs(&first, &changed).unwrap().equal);
        assert!(!plan_pairs(&first, &changed_keys).unwrap().equal);
        assert!(!plan_pairs(&first, &changed_count).unwrap().equal);
        let bounded_type = MapType::new(
            ValueType::String,
            ValueType::Integer,
            Some(CardinalityBound::at_most(2)),
        )
        .unwrap();
        let bounded = from_admitted_map(
            bounded_type,
            [("b", 2_i64), ("a", 1_i64)]
                .iter()
                .map(|(key, value)| {
                    (
                        Value::String((*key).to_owned()),
                        Value::Integer((*value).into()),
                    )
                })
                .collect(),
        )
        .unwrap();
        assert!(!plan_pairs(&first, &bounded).unwrap().equal);
        assert!(ValueType::map(map_type.clone()).admits(&first));
        assert!(!ValueType::map(
            MapType::new(ValueType::String, ValueType::Boolean, None).unwrap()
        )
        .admits(&first));
        let Value::Map(empty) = empty else {
            panic!("empty map constructor returns Value::Map");
        };
        assert_eq!(empty.map_type(), &map_type);
        assert!(empty.entries().is_empty());
        let Value::Map(first_map) = first else {
            panic!("map constructor returns Value::Map");
        };
        assert_eq!(first_map.entries().len(), 2);
        assert!(matches!(&first_map.entries()[0].0, Value::String(key) if key == "b"));
    }

    /// A repeated admitted key is refused at the later source entry.
    #[trace("TC-958", "FR-410-AC-2")]
    #[test]
    fn map_construction_refuses_duplicate_keys_deterministically() {
        let map_type = MapType::new(ValueType::String, ValueType::Integer, None).unwrap();
        let refusal = from_admitted_map(
            map_type,
            vec![
                (
                    Value::String("same".to_owned()),
                    Value::Integer(1_i64.into()),
                ),
                (
                    Value::String("other".to_owned()),
                    Value::Integer(2_i64.into()),
                ),
                (
                    Value::String("same".to_owned()),
                    Value::Integer(3_i64.into()),
                ),
            ],
        )
        .expect_err("duplicate map key must be rejected");

        assert_eq!(refusal.component, crate::value::Component::Element(2));
        assert_eq!(
            refusal.cause,
            crate::value::ConstructionCause::DuplicateMember
        );
        let refusal = from_admitted_map(
            MapType::new(ValueType::String, ValueType::Integer, None).unwrap(),
            [("x", 1_i64), ("y", 2_i64), ("y", 3_i64), ("x", 4_i64)]
                .iter()
                .map(|(key, value)| {
                    (
                        Value::String((*key).to_owned()),
                        Value::Integer((*value).into()),
                    )
                })
                .collect(),
        )
        .expect_err("report the smallest source index with an earlier equal key");
        assert_eq!(refusal.component, crate::value::Component::Element(2));

        use crate::ieee::{FloatType, IeeeValue, IeeeWidth};
        let float_type = ValueType::Float(FloatType::exact(IeeeWidth::Binary64));
        let float_map_type = MapType::new(float_type, ValueType::Integer, None).unwrap();
        let zeros = from_admitted_map(
            float_map_type.clone(),
            [0x8000_0000_0000_0000, 0]
                .into_iter()
                .map(|bits| {
                    (
                        Value::Float(IeeeValue::binary64(bits)),
                        Value::Integer(1_i64.into()),
                    )
                })
                .collect(),
        )
        .expect("signed zeros are distinct membership keys");
        assert!(matches!(&zeros, Value::Map(map) if map.entries().len() == 2));
        let nan = Value::Float(IeeeValue::binary64(0x7ff8_0000_0000_0001));
        let refusal = from_admitted_map(
            float_map_type,
            vec![
                (nan.clone(), Value::Integer(1_i64.into())),
                (nan, Value::Integer(2_i64.into())),
            ],
        )
        .expect_err("same NaN membership key must be refused");
        assert_eq!(refusal.component, crate::value::Component::Element(1));
    }

    /// The map matcher uses a sort-and-merge key comparison budget, not a
    /// nested scan whose comparison count grows quadratically.
    #[trace("TC-958", "FR-410-AC-6")]
    #[test]
    fn map_entry_matching_scales_by_sorting_key_order() {
        const ENTRY_COUNT: usize = 1_024;
        let left: Vec<_> = (0..ENTRY_COUNT)
            .rev()
            .map(|key| (Value::Integer(Integer::from(key)), Value::Boolean(true)))
            .collect();
        let right: Vec<_> = (0..ENTRY_COUNT)
            .map(|key| (Value::Integer(Integer::from(key)), Value::Boolean(true)))
            .collect();
        let comparisons = core::cell::Cell::new(0_usize);
        let mut compare_order = |left: &Value, right: &Value| {
            comparisons.set(comparisons.get() + 1);
            compare_map_membership_keys(left, right)
        };
        let mut compare_equal = |left: &Value, right: &Value| {
            comparisons.set(comparisons.get() + 1);
            compare_map_keys(left, right)
        };

        let matches = map_entry_matches_by(&left, &right, &mut compare_order, &mut compare_equal)
            .expect("integer map keys have total canonical keys");
        assert_eq!(matches.as_ref().map(Vec::len), Some(ENTRY_COUNT));
        assert!(
            comparisons.get() < ENTRY_COUNT * 32,
            "{} key comparisons exceeded the deterministic sort budget",
            comparisons.get()
        );
        let map_type = MapType::new(ValueType::Integer, ValueType::Boolean, None).unwrap();
        let left_map = from_admitted_map(map_type.clone(), left).unwrap();
        let right_map = from_admitted_map(map_type, right).unwrap();
        assert!(plan_pairs(&left_map, &right_map).unwrap().equal);
    }

    /// Map construction rejects a key outside K at its source entry.
    #[trace("TC-958", "FR-410-AC-3")]
    #[test]
    fn map_construction_rejects_wrong_key_type() {
        let refusal = from_admitted_map(
            MapType::new(ValueType::String, ValueType::Integer, None).unwrap(),
            vec![
                (Value::String("a".to_owned()), Value::Integer(1_i64.into())),
                (Value::Integer(7_i64.into()), Value::Integer(2_i64.into())),
            ],
        )
        .expect_err("Integer is not admitted by String key type");
        assert_eq!(refusal.component, crate::value::Component::Element(1));
        assert_eq!(refusal.cause, crate::value::ConstructionCause::TypeMismatch);
    }

    /// Map admission includes type identity, entry count and each K/V pair.
    #[trace("TC-958", "FR-410-AC-4")]
    #[test]
    fn map_admission_checks_bound_key_value_and_kind() {
        let bounded_type = MapType::new(
            ValueType::String,
            ValueType::Integer,
            Some(CardinalityBound::at_most(1)),
        )
        .unwrap();
        let declared = ValueType::map(bounded_type.clone());
        let one = from_admitted_map(
            bounded_type.clone(),
            vec![(Value::String("a".to_owned()), Value::Integer(1_i64.into()))],
        )
        .unwrap();
        assert!(declared.admits(&one));

        let two = from_admitted_map(
            bounded_type.clone(),
            ["a", "b"]
                .into_iter()
                .enumerate()
                .map(|(index, key)| {
                    (
                        Value::String(key.to_owned()),
                        Value::Integer((index as i64).into()),
                    )
                })
                .collect(),
        )
        .unwrap();
        assert!(!declared.admits(&two));

        let unbounded = from_admitted_map(
            MapType::new(ValueType::String, ValueType::Integer, None).unwrap(),
            vec![(Value::String("a".to_owned()), Value::Integer(1_i64.into()))],
        )
        .unwrap();
        assert!(!declared.admits(&unbounded));

        let wrong_values = from_admitted_map(
            bounded_type.clone(),
            vec![(Value::String("a".to_owned()), Value::Boolean(true))],
        )
        .unwrap();
        assert!(!declared.admits(&wrong_values));

        let sequence = from_admitted(
            CollectionType::new(CollectionKind::Sequence, ValueType::Integer, None),
            vec![Value::Integer(1_i64.into())],
        );
        assert!(!declared.admits(&sequence));
        let wrong_map_type = from_admitted_map(
            MapType::new(
                ValueType::String,
                ValueType::Boolean,
                Some(CardinalityBound::at_most(1)),
            )
            .unwrap(),
            vec![(Value::String("a".to_owned()), Value::Boolean(true))],
        )
        .unwrap();
        assert!(!declared.admits(&wrong_map_type));
        let map = from_admitted_map(
            bounded_type,
            vec![(Value::String("a".to_owned()), Value::Integer(1_i64.into()))],
        )
        .unwrap();
        for kind in CollectionKind::ALL {
            assert!(
                !ValueType::collection(CollectionType::new(kind, ValueType::Integer, None,))
                    .admits(&map)
            );
        }
    }

    /// Maps work as Set members and as map values.
    #[trace("TC-958", "FR-410-AC-8")]
    #[test]
    fn maps_form_sets_and_nest_as_map_values() {
        let inner_type = MapType::new(ValueType::String, ValueType::Integer, None).unwrap();
        let make_inner = |entries: &[(&str, i64)]| {
            from_admitted_map(
                inner_type.clone(),
                entries
                    .iter()
                    .map(|(key, value)| {
                        (
                            Value::String((*key).to_owned()),
                            Value::Integer((*value).into()),
                        )
                    })
                    .collect(),
            )
            .unwrap()
        };
        let first = make_inner(&[("b", 2), ("a", 1)]);
        let reordered = make_inner(&[("a", 1), ("b", 2)]);
        let smaller = make_inner(&[("a", 0)]);
        let set_type = CollectionType::new(
            CollectionKind::Set,
            ValueType::map(inner_type.clone()),
            None,
        );
        let set = form(
            &set_type,
            vec![first.clone(), reordered, smaller],
            &mut generous_meter(),
        )
        .completed()
        .expect("maps have membership keys");
        let Value::Collection(set) = set else {
            panic!("form returns a collection");
        };
        assert_eq!(set.elements().len(), 2);
        let Value::Map(smallest) = &set.elements()[0] else {
            panic!("set member is a map");
        };
        assert!(matches!(
            smallest.entries(),
            [(Value::String(key), Value::Integer(value))]
                if key == "a" && value == &Integer::zero()
        ));

        let outer_type =
            MapType::new(ValueType::String, ValueType::map(inner_type.clone()), None).unwrap();
        let outer_left = from_admitted_map(
            outer_type.clone(),
            vec![(Value::String("nested".to_owned()), first)],
        )
        .unwrap();
        let outer_right = from_admitted_map(
            outer_type,
            vec![(
                Value::String("nested".to_owned()),
                make_inner(&[("a", 1), ("b", 2)]),
            )],
        )
        .unwrap();
        assert!(plan_pairs(&outer_left, &outer_right).unwrap().equal);
        assert_eq!(
            compare_keys(&outer_left, &outer_right),
            Some(Ordering::Equal)
        );
    }

    /// Ordinary map equality uses numericEqual for float keys while map
    /// membership keeps signed zeros as distinct keys.
    #[trace("TC-958", "FR-410-AC-9")]
    #[test]
    fn float_map_equality_and_membership_use_different_key_relations() {
        use crate::ieee::{FloatType, IeeeValue, IeeeWidth};

        let map_type = MapType::new(
            ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
            ValueType::Integer,
            None,
        )
        .unwrap();
        let map_for = |bits| {
            from_admitted_map(
                map_type.clone(),
                vec![(
                    Value::Float(IeeeValue::binary64(bits)),
                    Value::Integer(1_i64.into()),
                )],
            )
            .unwrap()
        };
        let negative_zero = map_for(0x8000_0000_0000_0000);
        let positive_zero = map_for(0);

        assert!(plan_pairs(&negative_zero, &positive_zero).unwrap().equal);
        assert_ne!(
            compare_keys(&negative_zero, &positive_zero),
            Some(Ordering::Equal)
        );

        let set_type = CollectionType::new(CollectionKind::Set, ValueType::map(map_type), None);
        let set = form(
            &set_type,
            vec![negative_zero, positive_zero],
            &mut generous_meter(),
        )
        .completed()
        .expect("map membership keys distinguish signed zeros");
        let Value::Collection(set) = set else {
            panic!("form returns a collection");
        };
        assert_eq!(set.elements().len(), 2);
    }

    fn generous_limits() -> ScalarLimits {
        ScalarLimits {
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
        }
    }

    fn generous_meter() -> Meter {
        Meter::new(generous_limits())
    }

    fn e21_list_shape_operands(first_head_unequal: bool) -> (Value, Value) {
        let declaration = crate::node::NodeKey::fixture(42);
        let list = |heads: [i64; 8]| {
            heads
                .into_iter()
                .rev()
                .fold(None, |tail: Option<Value>, head| {
                    let tail = tail.map_or(FieldValue::Absent, FieldValue::Present);
                    Some(crate::value::from_admitted_slots(
                        declaration.clone(),
                        vec![
                            FieldValue::Present(Value::Integer(Integer::from(head))),
                            tail,
                        ]
                        .into_boxed_slice(),
                    ))
                })
                .expect("the qualification list has eight nodes")
        };
        let left_heads = if first_head_unequal {
            [0, 2, 3, 4, 5, 6, 7, 8]
        } else {
            [1, 2, 3, 4, 5, 6, 7, 8]
        };
        (list(left_heads), list([1, 2, 3, 4, 5, 6, 7, 8]))
    }

    /// `plan_equality` reports the exact pair count of two equal
    /// integers without charging.
    #[test]
    fn plan_equality_reports_pairs_without_charge() {
        let left = Value::Integer(Integer::one());
        let right = Value::Integer(Integer::one());
        let plan = plan_equality(&left, &right).unwrap();
        assert_eq!(plan.pair_events(), &Integer::one());
    }

    /// Structural equality uses IEEE numeric equality at float leaves: NaN is
    /// unequal to itself, while signed zero compares equal.
    #[trace("TC-194", "FR-149-AC-25")]
    #[test]
    fn float_leaves_use_ieee_numeric_equality() {
        use crate::ieee::IeeeValue;

        let equal = |left, right| {
            plan_equality(&Value::Float(left), &Value::Float(right))
                .expect("same-width float equality has a plan")
                .pair_events()
                == &Integer::one()
                && {
                    let mut meter = generous_meter();
                    planned_equality(&Value::Float(left), &Value::Float(right), &mut meter)
                        .completed()
                        .expect("meter admits one pair")
                }
        };
        assert!(equal(
            IeeeValue::binary64(0),
            IeeeValue::binary64(0x8000_0000_0000_0000)
        ));
        assert!(!equal(
            IeeeValue::binary64(0x7ff8_0000_0000_0000),
            IeeeValue::binary64(0x7ff8_0000_0000_0000)
        ));
    }

    /// Sequences recurse with numeric float equality, while set members use
    /// bit identity for both signed zero and NaN payloads.
    #[trace("TC-194", "FR-149-AC-25", "FR-149-AC-26")]
    #[test]
    fn collection_float_equality_selects_its_membership_relation() {
        use crate::collection::{
            form_collection, CardinalityBound, CollectionKind, CollectionType,
        };
        use crate::ieee::{FloatType, IeeeValue, IeeeWidth};
        use crate::value::ValueType;

        let make = |kind, bits: &[u64]| {
            let collection_type = CollectionType::new(
                kind,
                ValueType::Float(FloatType::exact(IeeeWidth::Binary64)),
                Some(CardinalityBound::new(0, 4).expect("valid bound")),
            );
            form_collection(
                &collection_type,
                bits.iter()
                    .copied()
                    .map(|bits| Value::Float(IeeeValue::binary64(bits)))
                    .collect(),
                &mut generous_meter(),
            )
            .expect("the completed values fit their bounds")
            .completed()
            .expect("the collection is admitted")
        };
        let equal = |left: &Value, right: &Value| {
            let mut meter = generous_meter();
            planned_equality(left, right, &mut meter)
                .completed()
                .expect("the equality plan fits the meter")
        };

        let positive_zero = 0;
        let negative_zero = 0x8000_0000_0000_0000;
        let nan = 0x7ff8_0000_0000_0000;
        let distinct_nan = 0x7ff8_0000_0000_0001;
        assert!(equal(
            &make(CollectionKind::Sequence, &[positive_zero]),
            &make(CollectionKind::Sequence, &[negative_zero])
        ));
        assert!(equal(
            &make(CollectionKind::Set, &[nan]),
            &make(CollectionKind::Set, &[nan])
        ));
        assert!(!equal(
            &make(CollectionKind::Set, &[positive_zero]),
            &make(CollectionKind::Set, &[negative_zero])
        ));
        assert!(!equal(
            &make(CollectionKind::Set, &[nan]),
            &make(CollectionKind::Set, &[distinct_nan])
        ));
    }

    /// TC-992: native String equality compares exact scalar sequences; its
    /// NFC control keeps Text's declared normalization semantics.
    #[trace("TC-992", "FR-056-AC-23")]
    #[test]
    fn native_string_equality_does_not_normalize_like_text() {
        let composed = Value::String("é".to_owned());
        let decomposed = Value::String("e\u{301}".to_owned());
        let equal_copy = Value::String("e\u{301}".to_owned());
        let compare = |left: &Value, right: &Value| {
            let mut meter = generous_meter();
            planned_equality(left, right, &mut meter)
                .completed()
                .expect("same native String type compares")
        };

        assert!(compare(&decomposed, &equal_copy));
        assert!(!compare(&composed, &decomposed));
        let text_type = crate::text::TextType::new(0, 8, crate::text::TextProfile::Nfc)
            .expect("valid text bounds");
        let text_value = |source: &str| {
            let mut meter = generous_meter();
            Value::Text(
                crate::text::admit_text(
                    &crate::text::TextPayload::from_utf8(source.as_bytes()).expect("UTF-8"),
                    &text_type,
                    &mut meter,
                )
                .completed()
                .expect("NFC text admission"),
            )
        };
        assert!(compare(&text_value("é"), &text_value("e\u{301}")));
    }

    /// `planned_equality` completes true for two equal integers
    /// under a generous meter, and a meter with no `value_occurrences` left
    /// cannot admit `equality.plan-form`, so the identical comparison
    /// returns `Outcome::Incomplete` instead.
    #[test]
    fn planned_equality_charges_and_a_tight_meter_is_incomplete() {
        let left = Value::Integer(Integer::one());
        let right = Value::Integer(Integer::one());

        let mut generous = generous_meter();
        let outcome = planned_equality(&left, &right, &mut generous);
        assert!(outcome.completed().expect("charges available"));

        let mut tight = Meter::new(ScalarLimits {
            value_occurrences: 0,
            ..generous_limits()
        });
        let outcome = planned_equality(&left, &right, &mut tight);
        assert!(matches!(outcome, Outcome::Incomplete(_)));
    }

    #[trace("TC-932", "FR-375-AC-9")]
    #[test]
    fn kernel_result_bearing_evaluation_returns_relation_plan_and_charge_order() {
        for (first_head_unequal, expected) in [(false, true), (true, false)] {
            let (left, right) = e21_list_shape_operands(first_head_unequal);
            assert_eq!(left.occ(), Integer::from(16_u64));
            assert_eq!(right.occ(), Integer::from(16_u64));
            let plan = plan_equality(&left, &right).unwrap();
            assert_eq!(plan.pair_events(), &Integer::from(17_u64));

            let mut meter = generous_meter();
            let Outcome::Completed(result) = planned_equality_with_plan(&left, &right, &mut meter)
            else {
                panic!("equality schedule should complete with generous limits");
            };
            assert_eq!(result.equal(), expected);
            assert_eq!(result.plan().pair_events(), &Integer::from(17_u64));
            assert_eq!(meter.admission_count(), 20);
            assert_eq!(meter.consumed(LimitKind::WorkUnits), 51);
            assert_eq!(meter.consumed(LimitKind::ResultUnits), 1);
            assert_eq!(meter.consumed(LimitKind::ValueOccurrences), 17);
            #[cfg(feature = "test-support")]
            {
                let mut expected_points =
                    vec![ChargePoint::EqualityPlanForm, ChargePoint::EqualityPlan];
                expected_points.extend([ChargePoint::EqualityPair; 17]);
                expected_points.push(ChargePoint::EqualityResultRetain);
                let actual_points: Vec<_> = meter
                    .admitted_charges()
                    .iter()
                    .map(|charge| charge.point)
                    .collect();
                assert_eq!(actual_points, expected_points);
            }

            // The production meter has no log. Denying result retention
            // exposes the admitted prefix through its public counters: both
            // setup charges and all 17 pair charges must precede the denial.
            let denial = InjectedDenial {
                point: ChargePoint::EqualityResultRetain,
                occurrence: NonZeroU64::new(1).unwrap(),
            };
            let mut stopped_meter = Meter::new(generous_limits()).with_injected_denial(denial);
            let stopped = planned_equality_with_plan(&left, &right, &mut stopped_meter);
            assert!(matches!(stopped, Outcome::Incomplete(ref incomplete)
                if incomplete.charge_point == ChargePoint::EqualityResultRetain));
            assert_eq!(stopped_meter.admission_count(), 19);
            assert_eq!(stopped_meter.consumed(LimitKind::WorkUnits), 50);
            assert_eq!(stopped_meter.consumed(LimitKind::ResultUnits), 0);
        }
    }

    #[test]
    fn plan_reservation_refusals_are_atomic_before_pair_charges() {
        let (left, right) = e21_list_shape_operands(true);
        for (limits, expected_kind, expected_limit, expected_next) in [
            (
                ScalarLimits {
                    work_units: 50,
                    ..generous_limits()
                },
                LimitKind::WorkUnits,
                50,
                19_u64,
            ),
            (
                ScalarLimits {
                    result_units: 0,
                    ..generous_limits()
                },
                LimitKind::ResultUnits,
                0,
                1_u64,
            ),
        ] {
            let mut meter = Meter::new(limits);
            let outcome = planned_equality_with_plan(&left, &right, &mut meter);
            assert!(matches!(outcome, Outcome::Incomplete(ref incomplete)
                if incomplete.limit_kind == expected_kind
                    && incomplete.limit == expected_limit
                    && incomplete.consumed == if expected_kind == LimitKind::WorkUnits { 32 } else { 0 }
                    && incomplete.next_charge == Integer::from(expected_next)
                    && incomplete.charge_point == ChargePoint::EqualityPlan));
            assert_eq!(meter.admission_count(), 1);
            assert_eq!(meter.consumed(LimitKind::WorkUnits), 32);
            assert_eq!(meter.consumed(LimitKind::ResultUnits), 0);
            assert_eq!(meter.consumed(LimitKind::ValueOccurrences), 16);
        }
    }

    #[cfg(feature = "test-support")]
    #[trace("TC-932", "FR-375-AC-10")]
    #[test]
    fn pair_and_result_refusals_stop_at_the_first_denied_charge() {
        let (left, right) = e21_list_shape_operands(true);
        let pair_denial = InjectedDenial {
            point: ChargePoint::EqualityPair,
            occurrence: NonZeroU64::new(17).unwrap(),
        };
        let mut pair_meter = Meter::new(generous_limits()).with_injected_denial(pair_denial);
        let pair_outcome = planned_equality_with_plan(&left, &right, &mut pair_meter);
        assert!(matches!(pair_outcome, Outcome::Incomplete(ref incomplete)
            if incomplete.charge_point == ChargePoint::EqualityPair));
        assert_eq!(pair_meter.admission_count(), 18);
        assert_eq!(pair_meter.consumed(LimitKind::WorkUnits), 49);
        assert_eq!(pair_meter.consumed(LimitKind::ResultUnits), 0);
        let pair_points: Vec<_> = pair_meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect();
        let mut expected_pair_prefix =
            vec![ChargePoint::EqualityPlanForm, ChargePoint::EqualityPlan];
        expected_pair_prefix.extend([ChargePoint::EqualityPair; 16]);
        assert_eq!(pair_points, expected_pair_prefix);

        let retain_denial = InjectedDenial {
            point: ChargePoint::EqualityResultRetain,
            occurrence: NonZeroU64::new(1).unwrap(),
        };
        let mut retain_meter = Meter::new(generous_limits()).with_injected_denial(retain_denial);
        let retain_outcome = planned_equality_with_plan(&left, &right, &mut retain_meter);
        assert!(matches!(retain_outcome, Outcome::Incomplete(ref incomplete)
            if incomplete.charge_point == ChargePoint::EqualityResultRetain));
        assert_eq!(retain_meter.admission_count(), 19);
        assert_eq!(retain_meter.consumed(LimitKind::WorkUnits), 50);
        assert_eq!(retain_meter.consumed(LimitKind::ResultUnits), 0);
        let retain_points: Vec<_> = retain_meter
            .admitted_charges()
            .iter()
            .map(|charge| charge.point)
            .collect();
        let mut expected_retain_prefix = expected_pair_prefix;
        expected_retain_prefix.push(ChargePoint::EqualityPair);
        assert_eq!(retain_points, expected_retain_prefix);
    }

    /// two equal integers plan one pair and compare equal.
    #[test]
    fn equal_integers_plan_one_equal_pair() {
        let left = Value::Integer(Integer::one());
        let right = Value::Integer(Integer::one());
        let plan = plan_pairs(&left, &right).unwrap();
        assert_eq!(plan.pairs, Integer::one());
        assert!(plan.equal);
    }

    /// TC-297 (FR-089-AC-6): a population pair is not an equality operand
    /// pair in the kernel.
    #[trace("TC-297", "FR-089-AC-6", "FR-369-AC-2", "FR-389-AC-8")]
    #[test]
    fn plan_pairs_refuses_a_population_pair() {
        use crate::identity::PopulationId;

        let left = Value::Population(PopulationId::fixture(1));
        let right = Value::Population(PopulationId::fixture(2));
        assert!(matches!(
            plan_pairs(&left, &right),
            Err(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::PopulationPair,
            })
        ));
        assert!(matches!(
            planned_equality(&left, &right, &mut Meter::new(generous_limits())),
            Outcome::Refused(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::PopulationPair,
            })
        ));
        assert!(matches!(
            plan_pairs(&left, &left),
            Err(Refusal::CheckedInvariant {
                cause: CheckedInvariantCause::PopulationPair,
            })
        ));
    }

    /// Trace: FR-369-AC-2
    #[test]
    fn incompatible_pairs_report_specific_causes_without_a_boolean() {
        use crate::collection::{from_admitted, CollectionKind, CollectionType};
        use crate::identity::PopulationId;
        use crate::value::ValueType;

        let collection =
            |kind| from_admitted(CollectionType::new(kind, ValueType::Integer, None), vec![]);
        let population = || Value::Population(PopulationId::fixture(1));
        let cases = [
            (
                collection(CollectionKind::Set),
                collection(CollectionKind::Bag),
                CheckedInvariantCause::CollectionKindMismatch,
            ),
            (
                population(),
                Value::Boolean(true),
                CheckedInvariantCause::ValueKindMismatch,
            ),
            (
                Value::Boolean(true),
                population(),
                CheckedInvariantCause::ValueKindMismatch,
            ),
            (
                Value::Integer(Integer::one()),
                Value::Boolean(true),
                CheckedInvariantCause::ValueKindMismatch,
            ),
        ];
        for (left, right, cause) in cases {
            let expected = Refusal::CheckedInvariant { cause };
            assert!(matches!(plan_pairs(&left, &right), Err(actual) if actual == expected));
            assert_eq!(plan_equality(&left, &right), Err(expected.clone()));
            assert_eq!(
                planned_equality(&left, &right, &mut Meter::new(generous_limits())),
                Outcome::Refused(expected)
            );
        }
    }

    /// a reference pair of different universes refuses with
    /// `ForeignReference` rather than comparing structurally, carrying the
    /// left operand's universe as `required` and the right's as `supplied`.
    #[test]
    fn foreign_reference_pair_is_refused() {
        use crate::identity::{EffectiveId, ObjectId, UniverseId};
        use crate::reference::ObjectReference;

        let object_type = EffectiveId::fixture(1);
        let object = ObjectId::new("o1").unwrap();
        let left_universe = UniverseId::fixture(10);
        let right_universe = UniverseId::fixture(20);
        let left = Value::Reference(ObjectReference::new(
            left_universe.clone(),
            object_type.clone(),
            object.clone(),
        ));
        let right = Value::Reference(ObjectReference::new(
            right_universe.clone(),
            object_type,
            object,
        ));
        let Err(refusal) = plan_pairs(&left, &right) else {
            panic!("expected ForeignReference");
        };
        assert_eq!(
            refusal,
            Refusal::ForeignReference {
                required: left_universe,
                supplied: right_universe,
            }
        );
    }
}
