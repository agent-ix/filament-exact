// SPDX-License-Identifier: AGPL-3.0-or-later
//! The type-owned total canonical key.
//!
//! Keys provide canonical ordering for values whose relation supports one;
//! they are not an ordering operator. A key exists only when equal values
//! under that relation share a key. Comparison is iterative, so value depth
//! never reaches the host stack. This API is for canonical collection order,
//! not a general-purpose map-key encoding.
//!
//! Two composites of different declarations are unequal under the equality
//! plan, so their keys differ: they order by the ascending bytes of the two
//! 32-byte declaration keys, before any slot. The order is total and has no
//! relation to ancestry, declaration names or slot contents. Two composites of
//! one declaration continue by slots in declaration order.
//!
//! Float equality is IEEE `numericEqual`, so NaN is unequal to itself and
//! ordinary float values have no key. Collection membership has a separate
//! bit-identity relation. Its float leaves are ordered by IEEE `totalOrder`,
//! which gives each distinct bit pattern a distinct position for set, bag and
//! ordered-set canonicalization. That membership key does not define ordinary
//! float equality or a map key.
//!
//! `Value::Enum` is [`crate::value::EnumMember`] here -- a bare
//! [`crate::identity::VariantId`] paired with its zero-based canonical rank --
//! not a declaration-aware value carrying a live position/case lookup.
//! QSpec-FR-144's enumeration key row (QSpec-FR-144-AC-9) fixes canonical order as
//! declaration position for an `ordered enum` and case-identifier byte order
//! otherwise; because `EnumMember::rank` is already that canonical-list index
//! (the shape that admitted it fixed it there,
//! `crate::value::EnumShape::rank`), comparing two same-enum members' ranks
//! numerically reproduces QSpec-FR-144's order for *both* cases at once, with no
//! declaration lookup and no case-name string needed here.

use alloc::{vec, vec::Vec};
use core::cmp::Ordering;

use crate::value::{FieldValue, Value};

/// One pending key comparison.
enum Task<'a> {
    Values(&'a Value, &'a Value, KeyRelation),
    Slots(&'a FieldValue, &'a FieldValue, KeyRelation),
    /// The proper-prefix tiebreak of two element lists whose common prefix
    /// has equal keys.
    Lengths(usize, usize),
}

#[derive(Clone, Copy)]
enum KeyRelation {
    Equality,
    Membership,
}

/// Compare the canonical keys of two values of one declared type. Two
/// composites of different declarations order by the ascending bytes of their
/// 32-byte declaration keys before any slot, because a slot of a declared
/// type can hold a conforming descendant (FR-377). `None` means the pair has
/// no key: unlike value variants, a composite against a non-composite, two
/// composites of one declaration with different slot counts, two enum members
/// of equal rank and different variants, a `Float`, or a population. `pub`:
/// an evaluator groups adjacent equal elements with this key before
/// [`crate::form_grouped`] and in `Contains`, so it must order exactly as
/// [`crate::form`]'s canonical sort does.
pub fn compare_keys(left: &Value, right: &Value) -> Option<Ordering> {
    compare_keys_with_relation(left, right, KeyRelation::Equality)
}

/// Compare canonical keys under the collection-membership relation, where
/// float leaves use bit identity rather than numeric equality.
pub(crate) fn compare_membership_keys(left: &Value, right: &Value) -> Option<Ordering> {
    compare_keys_with_relation(left, right, KeyRelation::Membership)
}

fn compare_keys_with_relation(
    left: &Value,
    right: &Value,
    relation: KeyRelation,
) -> Option<Ordering> {
    let mut tasks = vec![Task::Values(left, right, relation)];
    while let Some(task) = tasks.pop() {
        let ordering = match task {
            Task::Lengths(left, right) => left.cmp(&right),
            Task::Slots(left, right, relation) => match (left, right) {
                (FieldValue::Present(left), FieldValue::Present(right)) => {
                    tasks.push(Task::Values(left, right, relation));
                    continue;
                }
                (left, right) => slot_rank(left).cmp(&slot_rank(right)),
            },
            Task::Values(left, right, relation) => match leaf(left, right, relation, &mut tasks)? {
                Some(ordering) => ordering,
                None => continue,
            },
        };
        if ordering.is_ne() {
            return Some(ordering);
        }
    }
    Some(Ordering::Equal)
}

/// `absent` before `null` before a present value.
fn slot_rank(slot: &FieldValue) -> u8 {
    match slot {
        FieldValue::Absent => 0,
        FieldValue::Null => 1,
        FieldValue::Present(_) => 2,
    }
}

/// The ordering of a leaf pair, or `Some(None)` after scheduling the children
/// of a structured pair in comparison order.
fn leaf<'a>(
    left: &'a Value,
    right: &'a Value,
    relation: KeyRelation,
    tasks: &mut Vec<Task<'a>>,
) -> Option<Option<Ordering>> {
    let ordering = match (left, right) {
        (Value::Float(left), Value::Float(right))
            if matches!(relation, KeyRelation::Membership) && left.width() == right.width() =>
        {
            left.total_order_key().cmp(&right.total_order_key())
        }
        (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
        (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
        (Value::Rational(left), Value::Rational(right)) => left.cmp(right),
        (Value::Decimal(left), Value::Decimal(right)) => left.compare(right),
        (Value::Quantity(left), Value::Quantity(right)) if left.unit() == right.unit() => {
            left.magnitude().cmp(right.magnitude())
        }
        (Value::Text(left), Value::Text(right)) => {
            left.retained().as_bytes().cmp(right.retained().as_bytes())
        }
        (Value::String(left), Value::String(right)) => left.as_bytes().cmp(right.as_bytes()),
        (Value::Bytes(left), Value::Bytes(right)) => left.cmp(right),
        (Value::JsonObject(left), Value::JsonObject(right)) => left
            .canonical_text()
            .as_bytes()
            .cmp(right.canonical_text().as_bytes()),
        (Value::Uuid(left), Value::Uuid(right)) => {
            left.canonical_bytes().cmp(&right.canonical_bytes())
        }
        (Value::Timestamp(left), Value::Timestamp(right)) => left
            .canonical_text()
            .as_bytes()
            .cmp(right.canonical_text().as_bytes()),
        // A rank alone does not name a declaration. Two
        // members of *different* enum declarations can share a rank without
        // being equal under `crate::equality` (they have different
        // `VariantId`s), and this leaf carries no declaration to guard on.
        // When ranks are equal but the `VariantId`s differ, the pair is not
        // one keyed type, so this returns `None` rather than reporting a
        // false `Equal` -- the same contract `crate::equality`'s leaf match
        // already enforces on a `VariantId` mismatch.
        (Value::Enum(left), Value::Enum(right)) => {
            let ordering = left.rank().cmp(&right.rank());
            if ordering.is_eq() && left.variant() != right.variant() {
                return None;
            }
            ordering
        }
        (Value::Reference(left), Value::Reference(right)) => left.cmp(right),
        (Value::Option(left), Value::Option(right)) => match (left.payload(), right.payload()) {
            (Some(left), Some(right)) => {
                tasks.push(Task::Values(left, right, relation));
                return Some(None);
            }
            (left, right) => left.is_some().cmp(&right.is_some()),
        },
        // Composites of different declarations order by their structural
        // declaration keys, before any slot (FR-377).
        (Value::Composite(left), Value::Composite(right))
            if left.declaration() != right.declaration() =>
        {
            left.declaration().cmp(right.declaration())
        }
        (Value::Composite(left), Value::Composite(right))
            if left.slots().len() == right.slots().len() =>
        {
            let slots = left.slots().iter().zip(right.slots()).rev();
            tasks.extend(slots.map(|(left, right)| Task::Slots(left, right, relation)));
            return Some(None);
        }
        (Value::Union(left), Value::Union(right)) if left.declaration() == right.declaration() => {
            let ordering = left
                .member()
                .identifier()
                .as_bytes()
                .cmp(right.member().identifier().as_bytes());
            if ordering.is_eq() {
                if left.variant() != right.variant()
                    || left.payload().len() != right.payload().len()
                {
                    return None;
                }
                tasks.extend(
                    left.payload()
                        .iter()
                        .zip(right.payload())
                        .rev()
                        .map(|(left, right)| Task::Values(left, right, relation)),
                );
                return Some(None);
            }
            if left.variant() == right.variant() {
                return None;
            }
            ordering
        }
        (Value::Collection(left), Value::Collection(right)) => {
            let relation =
                if left.collection_type().kind() == crate::collection::CollectionKind::Sequence {
                    relation
                } else {
                    KeyRelation::Membership
                };
            let (left, right) = (left.elements(), right.elements());
            tasks.push(Task::Lengths(left.len(), right.len()));
            let elements = left.iter().zip(right).rev();
            tasks.extend(elements.map(|(left, right)| Task::Values(left, right, relation)));
            return Some(None);
        }
        (Value::Map(left), Value::Map(right)) => {
            let left_entries = ordered_map_entries(left.entries())?;
            let right_entries = ordered_map_entries(right.entries())?;
            tasks.push(Task::Lengths(left_entries.len(), right_entries.len()));
            for (left, right) in left_entries.iter().zip(&right_entries).rev() {
                tasks.push(Task::Values(&left.1, &right.1, relation));
                tasks.push(Task::Values(&left.0, &right.0, KeyRelation::Membership));
            }
            return Some(None);
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
            | Value::Bytes(_)
            | Value::JsonObject(_)
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
        ) => return None,
    };
    Some(Some(ordering))
}

/// Return map entries in ascending canonical key order. Map keys use the
/// membership key so float keys retain their total-order identity.
fn ordered_map_entries(entries: &[(Value, Value)]) -> Option<Vec<&(Value, Value)>> {
    let mut ordered: Vec<_> = entries.iter().collect();
    let mut incomparable = false;
    ordered.sort_unstable_by(
        |left, right| match compare_membership_keys(&left.0, &right.0) {
            Some(ordering) => ordering,
            None => {
                incomparable = true;
                Ordering::Equal
            }
        },
    );
    (!incomparable).then_some(ordered)
}

#[cfg(test)]
mod tests {
    use alloc::borrow::ToOwned;

    use ix_trace_rs::trace;

    use super::*;
    use crate::collection::{from_admitted_map, MapType};
    use crate::integer::Integer;
    #[cfg(feature = "json-object")]
    use crate::JsonObject;

    /// two equal integers key-compare equal; unequal integers key
    /// in the same order as their value ordering.
    #[test]
    fn integer_keys_follow_value_ordering() {
        let one = Value::Integer(Integer::one());
        let two = Value::Integer(Integer::one().add(&Integer::one()));
        assert_eq!(compare_keys(&one, &one), Some(Ordering::Equal));
        assert_eq!(compare_keys(&one, &two), Some(Ordering::Less));
    }

    /// TC-958: maps receive canonical keys independent of retained entry order.
    #[trace("TC-958", "FR-410-AC-7")]
    #[test]
    fn map_keys_sort_entries_by_key_and_compare_values() {
        let map_type = MapType::new(
            crate::value::ValueType::String,
            crate::value::ValueType::Integer,
            None,
        )
        .expect("leaf key type");
        let map = |entries: &[(&str, i64)]| {
            from_admitted_map(
                map_type.clone(),
                entries
                    .iter()
                    .map(|(key, value)| {
                        (
                            Value::String((*key).to_owned()),
                            Value::Integer(Integer::from(*value)),
                        )
                    })
                    .collect(),
            )
            .expect("unique map keys")
        };

        let first = map(&[("b", 2), ("a", 1)]);
        let reordered = map(&[("a", 1), ("b", 2)]);
        let changed = map(&[("a", 1), ("b", 3)]);
        assert_eq!(compare_keys(&first, &reordered), Some(Ordering::Equal));
        assert_eq!(compare_keys(&first, &changed), Some(Ordering::Less));
        assert_eq!(
            compare_keys(&map(&[("a", 9)]), &map(&[("b", 0)])),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_keys(&map(&[("a", 1)]), &map(&[("a", 1), ("b", 0)])),
            Some(Ordering::Less)
        );
        let sequence = crate::collection::from_admitted(
            crate::collection::CollectionType::new(
                crate::collection::CollectionKind::Sequence,
                crate::value::ValueType::String,
                None,
            ),
            vec![Value::String("a".to_owned())],
        );
        assert_eq!(compare_keys(&first, &sequence), None);
    }

    /// TC-992: native String keys preserve exact UTF-8 scalar order and do
    /// not alias the key of the same bytes under profile-bounded Text.
    #[trace("TC-992", "FR-056-AC-23")]
    #[test]
    fn native_string_keys_preserve_scalar_sequence_identity() {
        let composed = Value::String("é".to_owned());
        let decomposed = Value::String("e\u{301}".to_owned());
        let decomposed_copy = Value::String("e\u{301}".to_owned());

        assert_eq!(
            compare_keys(&decomposed, &decomposed_copy),
            Some(Ordering::Equal)
        );
        assert_eq!(
            compare_keys(&composed, &decomposed),
            Some(Ordering::Greater)
        );
        let uuid = Value::Uuid(
            crate::Uuid::from_canonical_text("00112233-4455-6677-8899-aabbccddeeff")
                .expect("canonical UUID"),
        );
        assert_eq!(compare_keys(&decomposed, &uuid), None);
    }

    /// FR-149-AC-24: native scalar keys use their canonical payload order.
    #[trace("FSpec-FR-149-AC-23", "FSpec-FR-149-AC-24")]
    #[test]
    #[cfg(feature = "json-object")]
    fn bytes_and_json_object_keys_follow_canonical_payload_order() {
        let empty_bytes = Value::Bytes(Vec::new());
        let one_zero = Value::Bytes(vec![0x00]);
        let two_zeros = Value::Bytes(vec![0x00, 0x00]);
        let ten = Value::Bytes(vec![0x10]);
        let two_thousand = Value::Bytes(vec![0x20, 0x00]);
        assert_eq!(compare_keys(&empty_bytes, &one_zero), Some(Ordering::Less));
        assert_eq!(compare_keys(&one_zero, &two_zeros), Some(Ordering::Less));
        assert_eq!(compare_keys(&ten, &two_thousand), Some(Ordering::Less));

        let array = Value::JsonObject(JsonObject::from_json_text("[]").unwrap());
        let object = Value::JsonObject(JsonObject::from_json_text("{}").unwrap());
        let array_copy = Value::JsonObject(JsonObject::from_json_text("[]").unwrap());
        assert_eq!(compare_keys(&array, &object), Some(Ordering::Less));
        assert_eq!(compare_keys(&array, &array_copy), Some(Ordering::Equal));
        assert_eq!(compare_keys(&one_zero, &array), None);
    }

    /// comparing values of unrelated types (a checked program never
    /// produces this) returns `None`, not a panic.
    #[test]
    fn mismatched_leaf_types_return_none() {
        let boolean = Value::Boolean(true);
        let integer = Value::Integer(Integer::one());
        assert_eq!(compare_keys(&boolean, &integer), None);
    }

    /// TC-297 (FR-089-AC-6): a population pair has no key in the kernel.
    #[trace("TC-297", "FR-089-AC-6")]
    #[test]
    fn compare_keys_yields_no_key_for_a_population_pair() {
        use crate::identity::PopulationId;

        let left = Value::Population(PopulationId::fixture(1));
        let right = Value::Population(PopulationId::fixture(2));
        assert_eq!(compare_keys(&left, &right), None);
    }

    /// `compare_keys` returns no key for a same-type `Value::Float` pair: a
    /// float's ordering is the total-order membership key of
    /// `compare_membership_keys`, which collections use, not an ordinary key.
    #[test]
    fn float_pair_has_no_key() {
        use crate::ieee::IeeeValue;

        let left = Value::Float(IeeeValue::binary64(0x3ff0_0000_0000_0000));
        let right = Value::Float(IeeeValue::binary64(0x3ff0_0000_0000_0000));
        assert_eq!(compare_keys(&left, &right), None);
    }

    /// Two members of one shape order by their rank, whatever the order of
    /// their `VariantId` structural order.
    #[trace("TC-409", "FR-088-AC-11", "FR-389-AC-8")]
    #[test]
    fn compare_keys_orders_members_of_one_shape_by_rank() {
        use crate::identity::VariantId;
        use crate::value::EnumMember;

        fn variant(byte: u8) -> VariantId {
            VariantId::fixture(byte)
        }

        // Rank 0 has the larger structural case name.
        let first = Value::Enum(EnumMember::new(variant(9), 0));
        let second = Value::Enum(EnumMember::new(variant(1), 1));
        assert_eq!(compare_keys(&first, &second), Some(Ordering::Less));
        assert_eq!(compare_keys(&second, &first), Some(Ordering::Greater));
        assert_eq!(compare_keys(&first, &first), Some(Ordering::Equal));
    }

    /// The kernel leaf carries no declaration to guard
    /// on, so two members of *different* enum declarations that happen to
    /// share a rank must still be told apart by `VariantId` -- they are not
    /// equal under `crate::equality`'s leaf match. Mutation proof: dropping
    /// the `left.variant() != right.variant()` guard back to a bare
    /// `left.rank().cmp(&right.rank())` makes this return
    /// `Some(Ordering::Equal)` instead of `None`.
    #[trace("TC-409", "FR-088-AC-11")]
    #[test]
    fn compare_keys_refuses_same_rank_different_declaration_members() {
        use crate::identity::VariantId;
        use crate::value::EnumMember;

        let left = Value::Enum(EnumMember::new(
            VariantId::new(crate::node::NodeKey::fixture(1), "Case").unwrap(),
            0,
        ));
        let right = Value::Enum(EnumMember::new(
            VariantId::new(crate::node::NodeKey::fixture(2), "Case").unwrap(),
            0,
        ));
        assert_eq!(compare_keys(&left, &right), None);
    }
}
