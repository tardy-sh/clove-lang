//! JSON <-> Clove Value conversion utilities

use crate::Value;

/// Convert serde_json::Value to Clove Value
pub fn json_to_clove(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else {
                Value::Float(n.as_f64().unwrap())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_clove).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, json_to_clove(v))).collect())
        }
    }
}

/// Convert Clove Value to serde_json::Value
pub fn clove_to_json(v: Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Integer(i) => serde_json::Value::Number(i.into()),
        Value::Float(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(clove_to_json).collect())
        }
        Value::Object(obj) => serde_json::Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, clove_to_json(v)))
                .collect(),
        ),
    }
}
