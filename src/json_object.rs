// SPDX-License-Identifier: AGPL-3.0-or-later
//! A JSON value retained as canonical RFC 8785 text.

use alloc::string::String;
use alloc::string::ToString;
#[cfg(feature = "json-object")]
use alloc::vec::Vec;
#[cfg(feature = "json-object")]
use core::cmp::Ordering;
use core::fmt;

/// Why JSON text could not become an admitted canonical JSON value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum JsonObjectError {
    /// The input is not a valid JSON value.
    #[error("invalid JSON text")]
    InvalidJson,
    /// Storage for canonical JSON text could not be reserved.
    #[error("storage for canonical JSON text could not be reserved")]
    Allocation,
}

/// A JSON value held in its canonical RFC 8785 spelling.
///
/// The payload is private: with the `json-object` feature, values can enter
/// through `from_json_text`, which parses and canonicalizes their text. Equal
/// JSON values therefore have equal retained bytes regardless of
/// object-member insertion order.
#[derive(Clone, Eq, PartialEq)]
pub struct JsonObject(String);

impl fmt::Debug for JsonObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl JsonObject {
    /// Parse and canonicalize one JSON value.
    ///
    /// # Errors
    ///
    /// [`JsonObjectError::InvalidJson`] for malformed JSON, or
    /// [`JsonObjectError::Allocation`] if output storage cannot be reserved.
    #[cfg(feature = "json-object")]
    pub fn from_json_text(text: &str) -> Result<Self, JsonObjectError> {
        use serde::Deserialize;

        let mut deserializer = serde_json::Deserializer::from_str(text);
        let value =
            ParsedJson::deserialize(&mut deserializer).map_err(|_| JsonObjectError::InvalidJson)?;
        deserializer
            .end()
            .map_err(|_| JsonObjectError::InvalidJson)?;
        canonical_text(&value).map(Self)
    }

    /// The canonical RFC 8785 spelling.
    pub fn canonical_text(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "json-object")]
enum Task<'a> {
    Value(&'a ParsedJson),
    String(&'a str),
    Number(&'a serde_json::Number),
    Raw(&'static str),
}

#[cfg(feature = "json-object")]
enum ParsedJson {
    Null,
    Boolean(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<ParsedJson>),
    Object(Vec<(String, ParsedJson)>),
}

#[cfg(feature = "json-object")]
impl<'de> serde::Deserialize<'de> for ParsedJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct JsonVisitor;

        impl<'de> serde::de::Visitor<'de> for JsonVisitor {
            type Value = ParsedJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("one JSON value with unique object member names")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(ParsedJson::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(ParsedJson::Null)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(ParsedJson::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(ParsedJson::Number(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(ParsedJson::Number(value.into()))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::Number::from_f64(value)
                    .map(ParsedJson::Number)
                    .ok_or_else(|| E::custom("JSON number is not finite"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(ParsedJson::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(ParsedJson::String(value))
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(ParsedJson::Array(values))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut members: Vec<(String, ParsedJson)> = Vec::new();
                while let Some((name, value)) = map.next_entry::<String, ParsedJson>()? {
                    members.push((name, value));
                }
                // Sort once, then check adjacent names. Scanning all prior
                // names for every member makes otherwise valid wide objects
                // quadratic to admit.
                members.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                if members.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(serde::de::Error::custom("duplicate JSON object member"));
                }
                Ok(ParsedJson::Object(members))
            }
        }

        deserializer.deserialize_any(JsonVisitor)
    }
}

#[cfg(feature = "json-object")]
fn push<T>(items: &mut Vec<T>, item: T) -> Result<(), JsonObjectError> {
    items
        .try_reserve(1)
        .map_err(|_| JsonObjectError::Allocation)?;
    items.push(item);
    Ok(())
}

#[cfg(feature = "json-object")]
fn append(output: &mut String, text: &str) -> Result<(), JsonObjectError> {
    output
        .try_reserve(text.len())
        .map_err(|_| JsonObjectError::Allocation)?;
    output.push_str(text);
    Ok(())
}

#[cfg(feature = "json-object")]
fn utf16_cmp(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

#[cfg(feature = "json-object")]
fn canonical_text(root: &ParsedJson) -> Result<String, JsonObjectError> {
    let mut output = String::new();
    let mut tasks = Vec::new();
    push(&mut tasks, Task::Value(root))?;
    while let Some(task) = tasks.pop() {
        match task {
            Task::Raw(text) => append(&mut output, text)?,
            Task::String(text) => {
                let escaped =
                    serde_json::to_string(text).map_err(|_| JsonObjectError::InvalidJson)?;
                append(&mut output, &escaped)?;
            }
            Task::Number(number) => {
                let value = number.as_f64().ok_or(JsonObjectError::InvalidJson)?;
                let mut buffer = ryu_js::Buffer::new();
                append(&mut output, buffer.format_finite(value))?;
            }
            Task::Value(value) => match value {
                ParsedJson::Null => append(&mut output, "null")?,
                ParsedJson::Boolean(false) => append(&mut output, "false")?,
                ParsedJson::Boolean(true) => append(&mut output, "true")?,
                ParsedJson::Number(number) => push(&mut tasks, Task::Number(number))?,
                ParsedJson::String(text) => push(&mut tasks, Task::String(text))?,
                ParsedJson::Array(values) => {
                    push(&mut tasks, Task::Raw("]"))?;
                    for (index, child) in values.iter().enumerate().rev() {
                        push(&mut tasks, Task::Value(child))?;
                        if index > 0 {
                            push(&mut tasks, Task::Raw(","))?;
                        }
                    }
                    push(&mut tasks, Task::Raw("["))?;
                }
                ParsedJson::Object(members) => {
                    let mut ordered: Vec<_> = members.iter().collect();
                    ordered.sort_by(|(left, _), (right, _)| utf16_cmp(left, right));
                    push(&mut tasks, Task::Raw("}"))?;
                    for (index, (name, child)) in ordered.iter().enumerate().rev() {
                        push(&mut tasks, Task::Value(child))?;
                        push(&mut tasks, Task::Raw(":"))?;
                        push(&mut tasks, Task::String(name))?;
                        if index > 0 {
                            push(&mut tasks, Task::Raw(","))?;
                        }
                    }
                    push(&mut tasks, Task::Raw("{"))?;
                }
            },
        }
    }
    Ok(output)
}

#[cfg(all(test, feature = "json-object"))]
mod tests {
    use super::{JsonObject, JsonObjectError};

    /// FR-149-AC-23/24: canonical JSON ignores member insertion order and
    /// uses RFC 8785 string, key and number forms.
    #[test]
    fn canonical_text_orders_keys_and_normalizes_json_spelling() {
        let object = JsonObject::from_json_text(r#"{"":1,"𐀀":-0.0,"escaped":"/\n"}"#)
            .expect("valid JSON value");
        assert_eq!(object.canonical_text(), r#"{"escaped":"/\n","𐀀":0,"":1}"#);
        assert_eq!(
            JsonObject::from_json_text("{").unwrap_err(),
            JsonObjectError::InvalidJson
        );
        assert_eq!(
            JsonObject::from_json_text(r#"{"a":1,"a":2}"#).unwrap_err(),
            JsonObjectError::InvalidJson
        );
    }
}
