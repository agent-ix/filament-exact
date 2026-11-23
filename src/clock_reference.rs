// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clock-place references: a `Reference<Clock>` value.
//!
//! A clock-place reference names one clock place by its universe and its
//! canonical `place/v2` place id text. It is compared, ordered, hashed and
//! sized by that pair alone; the kernel never resolves it. The constructor
//! checks the five model-free encoding rules of FR-415 Behavior 3 and nothing
//! else: the declared model owns every other rule of FSL FR-230 "Place
//! identity".

use alloc::string::String;

use crate::identity::UniverseId;

/// A clock-place reference: the outer universe and the canonical `place/v2`
/// place id text of the clock place it names.
///
/// Rust equality, hashing and ordering are those of the pair (universe, text
/// octets), and the order is the canonical key order of FR-415 Behavior 6. The
/// language defines no ordering operator over place ids; the order serves
/// canonical collection order and state keys only.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ClockPlaceReference {
    universe: UniverseId,
    text: String,
}

impl ClockPlaceReference {
    /// The universe that owns the named clock place.
    pub fn universe(&self) -> &UniverseId {
        &self.universe
    }

    /// The place id text, byte for byte as supplied to [`Self::new`].
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[cfg(feature = "json-object")]
impl ClockPlaceReference {
    /// Build the reference `(universe, text)` when `text` meets the five
    /// encoding rules of FR-415 Behavior 3 (a subset of the model-free rules
    /// of FSL FR-230 "Canonical text grammar"). The text is retained
    /// unchanged.
    ///
    /// # Errors
    ///
    /// `ConstructionRefusal { component: Component::Value, cause:
    /// ConstructionCause::PlaceIdNoncanonical }` for any other text.
    pub fn new(universe: UniverseId, text: &str) -> Result<Self, crate::ConstructionRefusal> {
        if is_canonical_place_id(text) {
            Ok(Self {
                universe,
                text: String::from(text),
            })
        } else {
            Err(crate::ConstructionRefusal {
                component: crate::Component::Value,
                cause: crate::ConstructionCause::PlaceIdNoncanonical,
            })
        }
    }
}

/// The five rules of FR-415 Behavior 3, in order.
#[cfg(feature = "json-object")]
fn is_canonical_place_id(text: &str) -> bool {
    use alloc::string::ToString;
    use alloc::vec::Vec;
    use serde_json::Value as Json;

    // Rule 1: `place/v2/` then exactly five nonempty `/`-separated components.
    let Some(rest) = text.strip_prefix("place/v2/") else {
        return false;
    };
    let components: Vec<&str> = rest.split('/').collect();
    let Ok([p, k, s, t, j]) = <[&str; 5]>::try_from(components) else {
        return false;
    };
    // Rules 2 and 3 per component, yielding the JSON that rules 4 and 5 read.
    let (Some(p), Some(k), Some(s), Some(t), Some(j)) = (
        decode_component(p),
        decode_component(k),
        decode_component(s),
        decode_component(t),
        decode_component(j),
    ) else {
        return false;
    };

    // Rule 4: `K` is a nonempty string; `P`, `S`, `T`, `J` are arrays; `T` is
    // not empty.
    let (Json::String(key), Json::Array(_), Json::Array(_), Json::Array(t), Json::Array(j)) =
        (k, p, s, t, j)
    else {
        return false;
    };
    if key.is_empty() || t.is_empty() {
        return false;
    }

    // Rule 5: `J` holds one `["slot", pathPosition, index]` per `collection`
    // entry of `T`, in order, with `pathPosition` the entry's position in `T`.
    let collections = t.iter().enumerate().filter(|(_, entry)| {
        entry
            .as_array()
            .and_then(|entry| entry.first())
            .and_then(Json::as_str)
            == Some("collection")
    });
    let mut slots = j.iter();
    for (position, _) in collections {
        let Some(slot) = slots.next() else {
            return false;
        };
        let Some([Json::String(tag), Json::String(path), Json::String(index)]) =
            slot.as_array().map(Vec::as_slice)
        else {
            return false;
        };
        if tag != "slot"
            || !is_decimal_spelling(path)
            || !is_decimal_spelling(index)
            || *path != position.to_string()
        {
            return false;
        }
    }
    slots.next().is_none()
}

/// Rules 2 and 3 for one component: lowercase even-length hexadecimal whose
/// octets are UTF-8 and one JSON value that is its own JCS serialization.
#[cfg(feature = "json-object")]
fn decode_component(component: &str) -> Option<serde_json::Value> {
    let digits = component.as_bytes();
    if digits.is_empty() || digits.len() % 2 != 0 {
        return None;
    }
    let mut octets = alloc::vec::Vec::with_capacity(digits.len() / 2);
    for pair in digits.chunks_exact(2) {
        octets.push((hex_value(pair[0])? << 4) | hex_value(pair[1])?);
    }
    let json_text = String::from_utf8(octets).ok()?;
    let canonical = crate::JsonObject::from_json_text(&json_text).ok()?;
    if canonical.canonical_text() != json_text {
        return None;
    }
    serde_json::from_str(&json_text).ok()
}

/// A lowercase hexadecimal digit's value.
#[cfg(feature = "json-object")]
fn hex_value(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

/// Whether `text` matches `0 | [1-9][0-9]*`.
#[cfg(feature = "json-object")]
fn is_decimal_spelling(text: &str) -> bool {
    text == "0"
        || text
            .strip_prefix(|first: char| first.is_ascii_digit() && first != '0')
            .is_some_and(|tail| tail.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(all(test, feature = "json-object"))]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec;
    use core::cmp::Ordering;

    use super::ClockPlaceReference;
    use crate::{
        compare_keys, form, from_admitted, from_admitted_map, plan_equality, planned_equality,
        CardinalityBound, CheckedInvariantCause, CollectionKind, CollectionType, Component,
        ConstructionCause, ConstructionRefusal, EffectiveId, Integer, JsonObject, MapType, Meter,
        ObjectId, ObjectReference, Outcome, Refusal, ScalarLimits, UniverseId, Value, ValueType,
    };

    const P: &str = r#"["source","a","i","ix://i/calls"]"#;
    const S: &str = r#"["source","a","i","ix://i/C"]"#;
    const T: &str = r#"[["field",["source","a","i","ix://i/C"],"ix://i/C/x",["clock"]]]"#;
    const TQ: &str = r#"[["field",["source","a","i","ix://i/C"],"ix://i/C/q",["collection","sequence","0",null,["clock"]]],["collection","sequence","0",null,["clock"]]]"#;
    /// The complete literal of FSL FR-230-AC-43.
    const A: &str = "place/v2/5b22736f75726365222c2261222c2269222c2269783a2f2f692f63616c6c73225d/22632f6422/5b22736f75726365222c2261222c2269222c2269783a2f2f692f43225d/5b5b226669656c64222c5b22736f75726365222c2261222c2269222c2269783a2f2f692f43225d2c2269783a2f2f692f432f78222c5b22636c6f636b225d5d5d/5b5d";

    fn hex(json: &str) -> String {
        json.bytes().map(|byte| format!("{byte:02x}")).collect()
    }

    /// `A` with the given JSON texts for its five components.
    fn place(p: &str, k: &str, s: &str, t: &str, j: &str) -> String {
        format!(
            "place/v2/{}/{}/{}/{}/{}",
            hex(p),
            hex(k),
            hex(s),
            hex(t),
            hex(j)
        )
    }

    fn a_with(k: &str) -> String {
        place(P, k, S, T, "[]")
    }

    fn u1() -> UniverseId {
        UniverseId::fixture(1)
    }

    fn u2() -> UniverseId {
        UniverseId::fixture(2)
    }

    fn make(universe: UniverseId, text: &str) -> ClockPlaceReference {
        ClockPlaceReference::new(universe, text).expect("a canonical place id")
    }

    fn val(universe: UniverseId, text: &str) -> Value {
        Value::ClockReference(make(universe, text))
    }

    fn refusal(text: &str) -> Result<ClockPlaceReference, ConstructionRefusal> {
        ClockPlaceReference::new(u1(), text)
    }

    fn limits() -> ScalarLimits {
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

    fn noncanonical() -> ConstructionRefusal {
        ConstructionRefusal {
            component: Component::Value,
            cause: ConstructionCause::PlaceIdNoncanonical,
        }
    }

    fn equal(left: &Value, right: &Value) -> Outcome<bool> {
        planned_equality(left, right, &mut Meter::new(limits()))
    }

    /// The fixture text is the FSL FR-230-AC-43 literal.
    #[test]
    fn the_fixture_builder_reproduces_the_fsl_literal() {
        assert_eq!(a_with(r#""c/d""#), A);
        assert_eq!(A.len(), 279);
    }

    /// Trace: FR-415-AC-1, TC-963
    #[test]
    fn only_the_clock_reference_type_admits_a_clock_reference() {
        let clock = val(u1(), A);
        let object_type = EffectiveId::fixture(6);
        assert!(ValueType::ClockReference.admits(&clock));
        for other in [
            ValueType::Reference(object_type.clone()),
            ValueType::String,
            ValueType::Bytes,
            ValueType::JsonObject,
        ] {
            assert!(!other.admits(&clock));
        }
        let object = Value::Reference(ObjectReference::new(
            u1(),
            object_type,
            ObjectId::new(A).unwrap(),
        ));
        let json = JsonObject::from_json_text(&format!("{:?}", A)).unwrap();
        for other in [
            object,
            Value::String(String::from(A)),
            Value::Bytes(A.as_bytes().to_vec()),
            Value::JsonObject(json),
        ] {
            assert!(!ValueType::ClockReference.admits(&other));
        }
        assert_eq!(ValueType::ClockReference, ValueType::ClockReference);
        assert_ne!(ValueType::ClockReference, ValueType::JsonObject);
    }

    /// Trace: FR-415-AC-1, TC-963
    #[test]
    fn wrapper_types_and_a_clock_keyed_map_admit_clock_references() {
        let clock = val(u1(), A);
        let option =
            crate::OptionValue::from_admitted(ValueType::ClockReference, Some(clock.clone()));
        assert!(
            ValueType::Option(alloc::sync::Arc::new(ValueType::ClockReference)).admits(&option)
        );

        let sequence_type = CollectionType::new(
            CollectionKind::Sequence,
            ValueType::ClockReference,
            Some(CardinalityBound::new(0, 3).unwrap()),
        );
        let sequence = from_admitted(sequence_type.clone(), vec![clock.clone()]);
        assert!(ValueType::collection(sequence_type).admits(&sequence));

        let values_type = MapType::new(ValueType::String, ValueType::ClockReference, None).unwrap();
        let by_string = from_admitted_map(
            values_type.clone(),
            vec![(Value::String("k".into()), clock)],
        )
        .unwrap();
        assert!(ValueType::map(values_type).admits(&by_string));

        let map_type = MapType::new(ValueType::ClockReference, ValueType::Integer, None)
            .expect("a clock reference is a leaf key type");
        let entry = |k: &str, n: i64| (val(u1(), &a_with(k)), Value::Integer(Integer::from(n)));
        let map = from_admitted_map(
            map_type.clone(),
            vec![entry(r#""c/e""#, 1), entry(r#""c/d""#, 2)],
        )
        .unwrap();
        assert!(ValueType::map(map_type).admits(&map));
        let Value::Map(map) = map else {
            panic!("a map value");
        };
        assert_eq!(map.entries().len(), 2);
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn the_canonical_texts_construct() {
        assert!(refusal(A).is_ok());
        let slot = place(P, r#""c/d""#, S, TQ, r#"[["slot","1","7"]]"#);
        assert!(refusal(&slot).is_ok());
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn rule_1_refuses_a_text_that_is_not_place_v2_and_five_nonempty_components() {
        let v1 = A.replacen("place/v2/", "place/v1/", 1);
        let components: alloc::vec::Vec<&str> = A.split('/').collect();
        let four = components[..components.len() - 1].join("/");
        let trailing = format!("{A}/");
        let empty_third = components
            .iter()
            .enumerate()
            .map(|(i, c)| if i == 4 { "" } else { c })
            .collect::<alloc::vec::Vec<_>>()
            .join("/");
        for text in [v1, four, trailing, empty_third, String::new()] {
            assert_eq!(refusal(&text), Err(noncanonical()), "{text}");
        }
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn rule_2_refuses_odd_length_and_non_lowercase_hex_components() {
        let components: alloc::vec::Vec<&str> = A.split('/').collect();
        let with_third = |third: String| {
            let mut parts: alloc::vec::Vec<String> =
                components.iter().map(|c| String::from(*c)).collect();
            parts[4] = third;
            parts.join("/")
        };
        let shortened = with_third(components[4][..components[4].len() - 1].into());
        let non_hex = with_third(components[4].replacen('5', "g", 1));
        for text in [shortened, non_hex] {
            assert_eq!(refusal(&text), Err(noncanonical()), "{text}");
        }
        let uppercase = A.replacen('b', "B", 1);
        assert_eq!(refusal(&uppercase), Err(noncanonical()));
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn rule_3_refuses_non_utf8_and_non_jcs_json_components() {
        let components: alloc::vec::Vec<&str> = A.split('/').collect();
        let mut non_utf8 = components.clone();
        non_utf8[2] = "ff";
        let mut spaced = components.clone();
        let spaced_j = hex("[ ]");
        spaced[6] = &spaced_j;
        let unsorted = place(P, r#""c/d""#, S, r#"[{"b":1,"a":1}]"#, "[]");
        for text in [non_utf8.join("/"), spaced.join("/"), unsorted] {
            assert_eq!(refusal(&text), Err(noncanonical()), "{text}");
        }
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn rule_4_refuses_a_wrong_json_kind_per_component() {
        for text in [
            a_with(r#""""#),
            a_with("1"),
            place(P, r#""c/d""#, S, "[]", "[]"),
            place(r#""x""#, r#""c/d""#, S, T, "[]"),
            place(P, r#""c/d""#, r#"{}"#, T, "[]"),
            place(P, r#""c/d""#, S, T, "{}"),
        ] {
            assert_eq!(refusal(&text), Err(noncanonical()), "{text}");
        }
    }

    /// Trace: FR-415-AC-2, TC-963
    #[test]
    fn rule_5_refuses_a_slot_list_that_does_not_match_the_collection_entries() {
        let key = r#""c/d""#;
        for (t, j) in [
            (T, r#"[["slot","0","0"]]"#),
            (T, r#"[["slot","01","0"]]"#),
            (TQ, "[]"),
            (TQ, r#"[["slot","0","7"]]"#),
            (TQ, r#"[["slot","1"]]"#),
            (TQ, r#"[["slots","1","7"]]"#),
            (TQ, r#"[["slot","1","07"]]"#),
            (TQ, r#"[["slot","1","7"],["slot","1","7"]]"#),
            (TQ, r#"[["slot","1",7]]"#),
        ] {
            assert_eq!(
                refusal(&place(P, key, S, t, j)),
                Err(noncanonical()),
                "{t} {j}"
            );
        }
    }

    /// Trace: FR-415-AC-3, TC-963
    #[test]
    fn a_value_round_trips_through_its_outputs() {
        let slot = place(P, r#""c/d""#, S, TQ, r#"[["slot","1","7"]]"#);
        for text in [A, slot.as_str()] {
            let original = make(u1(), text);
            assert_eq!(original.text(), text);
            assert_eq!(original.universe(), &u1());
            let again = make(original.universe().clone(), original.text());
            assert_eq!(again, original);
            let (left, right) = (
                Value::ClockReference(original),
                Value::ClockReference(again),
            );
            assert_eq!(equal(&left, &right), Outcome::Completed(true));
            assert_eq!(compare_keys(&left, &right), Some(Ordering::Equal));
        }
    }

    /// Trace: FR-415-AC-4, TC-963
    #[test]
    fn equality_is_by_place_text_and_refuses_equal_text_across_universes() {
        let boolean_plan = plan_equality(&Value::Boolean(true), &Value::Boolean(true));
        let base = val(u1(), A);
        let other_member = A.replacen(&hex(r#"ix://i/C/x"#), &hex(r#"ix://i/C/y"#), 1);
        assert_ne!(other_member, A);
        let other_population = place(
            r#"["source","a","i","ix://i/other"]"#,
            r#""c/d""#,
            S,
            T,
            "[]",
        );
        assert_eq!(equal(&base, &val(u1(), A)), Outcome::Completed(true));
        for text in [a_with(r#""c/e""#), other_member, other_population] {
            let other = val(u1(), &text);
            assert_eq!(equal(&base, &other), Outcome::Completed(false));
            assert_eq!(plan_equality(&base, &other), boolean_plan);
        }
        assert_eq!(plan_equality(&base, &base), boolean_plan);
        let elsewhere = val(u2(), &a_with(r#""c/e""#));
        assert_eq!(equal(&base, &elsewhere), Outcome::Completed(false));
        assert_eq!(plan_equality(&base, &elsewhere), boolean_plan);

        let foreign = val(u2(), A);
        let expected = Refusal::ForeignReference {
            required: u1(),
            supplied: u2(),
        };
        assert_eq!(expected.code(), Some("foreign_reference"));
        assert_eq!(expected.cause(), Some("foreign-universe"));
        assert_eq!(plan_equality(&base, &foreign), Err(expected.clone()));
        assert_eq!(equal(&base, &foreign), Outcome::Refused(expected));
    }

    /// Trace: FR-415-AC-4, TC-963
    #[test]
    fn a_clock_reference_paired_with_another_kind_is_a_kind_mismatch() {
        let base = val(u1(), A);
        let mismatch = Refusal::CheckedInvariant {
            cause: CheckedInvariantCause::ValueKindMismatch,
        };
        let object = Value::Reference(ObjectReference::new(
            u1(),
            EffectiveId::fixture(6),
            ObjectId::new(A).unwrap(),
        ));
        for other in [object, Value::String(String::from(A))] {
            assert_eq!(plan_equality(&base, &other), Err(mismatch.clone()));
            assert_eq!(plan_equality(&other, &base), Err(mismatch.clone()));
        }
    }

    /// Trace: FR-415-AC-5, TC-963
    #[test]
    fn keys_order_by_universe_then_place_text() {
        let key = |k: &str| val(u1(), &a_with(&format!("{k:?}")));
        assert_eq!(compare_keys(&key("c"), &key("c/d")), Some(Ordering::Less));
        assert_eq!(compare_keys(&key("c/d"), &key("c/e")), Some(Ordering::Less));
        let earlier_population = val(
            u1(),
            &place(r#"["source","a","i","ix://i/b"]"#, r#""c/e""#, S, T, "[]"),
        );
        assert_eq!(
            compare_keys(&earlier_population, &key("c/d")),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_keys(&val(u1(), A), &val(u2(), A)),
            Some(u1().cmp(&u2()))
        );
        assert_eq!(
            compare_keys(&val(u1(), A), &val(u1(), A)),
            Some(Ordering::Equal)
        );
        let object = Value::Reference(ObjectReference::new(
            u1(),
            EffectiveId::fixture(6),
            ObjectId::new(A).unwrap(),
        ));
        assert_eq!(compare_keys(&val(u1(), A), &object), None);
        assert_eq!(compare_keys(&object, &val(u1(), A)), None);
    }

    /// Trace: FR-415-AC-5, TC-963
    #[test]
    fn a_set_holds_clock_references_in_key_order_for_any_insertion_order() {
        let set_type = CollectionType::new(CollectionKind::Set, ValueType::ClockReference, None);
        let ordered = [
            val(
                u1(),
                &place(r#"["source","a","i","ix://i/b"]"#, r#""c/e""#, S, T, "[]"),
            ),
            val(u1(), &a_with(r#""c/d""#)),
            val(u1(), &a_with(r#""c/e""#)),
        ];
        for order in [[0, 1, 2], [2, 0, 1], [1, 2, 0]] {
            let members = order.iter().map(|&i| ordered[i].clone()).collect();
            let set = form(&set_type, members, &mut Meter::new(limits()))
                .completed()
                .expect("a set forms");
            let Value::Collection(set) = set else {
                panic!("a collection");
            };
            let held: alloc::vec::Vec<&ClockPlaceReference> = set
                .elements()
                .iter()
                .map(|member| match member {
                    Value::ClockReference(member) => member,
                    _ => panic!("a clock reference"),
                })
                .collect();
            let expected: alloc::vec::Vec<&ClockPlaceReference> = ordered
                .iter()
                .map(|member| match member {
                    Value::ClockReference(member) => member,
                    _ => unreachable!(),
                })
                .collect();
            assert_eq!(held, expected);
        }
    }

    /// Trace: FR-415-AC-6, TC-963
    #[test]
    fn a_clock_reference_counts_one_occurrence_and_its_text_length() {
        let value = val(u1(), A);
        assert_eq!(value.occ(), Integer::one());
        let Value::ClockReference(reference) = &value else {
            panic!("a clock reference");
        };
        assert_eq!(reference.text().len(), 279);
    }
}
