//! JSON output serialization for Clove query language values.
//!
//! This module provides JSON serialization with support for both compact and
//! pretty-printed output formats. All output is deterministic (object keys are
//! sorted) and follows standard JSON formatting rules.
//!
//! # Features
//!
//! - **Compact output** via [`to_json()`] - minimal whitespace for efficient transmission
//! - **Pretty output** via [`to_json_pretty()`] - human-readable with 2-space indentation
//! - **String escaping** - handles special characters, control codes, and Unicode
//! - **Type preservation** - maintains distinction between integers and floats
//! - **Deterministic** - object keys are always sorted alphabetically
//!
//! # Examples
//!
//! ```
//! use clove_lang::Value;
//! use clove_lang::output::{to_json, to_json_pretty};
//!
//! let value = Value::Integer(42);
//!
//! // Compact output
//! assert_eq!(to_json(&value), "42");
//!
//! // Pretty output (identical for simple values)
//! assert_eq!(to_json_pretty(&value), "42");
//! ```

use crate::value::Value;

pub struct JsonPrinter {
    pretty: bool,
}

impl JsonPrinter {
    pub fn new(pretty: bool) -> Self {
        JsonPrinter { pretty }
    }

    pub fn print(&self, value: &Value) -> String {
        self.print_value(value, 0)
    }

    fn print_value(&self, value: &Value, indent: usize) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::String(s) => {
                // Escape special characters
                format!("\"{}\"", self.escape_string(s))
            }
            Value::Array(arr) => self.print_array(arr, indent),
            Value::Object(obj) => self.print_object(obj, indent),
        }
    }

    fn print_array(&self, arr: &[Value], indent: usize) -> String {
        if arr.is_empty() {
            return "[]".to_string();
        }

        if self.pretty {
            let mut result = "[\n".to_string();
            let items: Vec<String> = arr
                .iter()
                .map(|v| {
                    format!(
                        "{}{}",
                        self.indent(indent + 1),
                        self.print_value(v, indent + 1)
                    )
                })
                .collect();
            result.push_str(&items.join(",\n"));
            result.push('\n');
            result.push_str(&self.indent(indent));
            result.push(']');
            result
        } else {
            let items: Vec<String> = arr.iter().map(|v| self.print_value(v, indent)).collect();
            format!("[{}]", items.join(","))
        }
    }

    fn print_object(
        &self,
        obj: &std::collections::HashMap<String, Value>,
        indent: usize,
    ) -> String {
        if obj.is_empty() {
            return "{}".to_string();
        }

        // Sort keys for deterministic output
        let mut keys: Vec<_> = obj.keys().collect();
        keys.sort();

        if self.pretty {
            let mut result = "{\n".to_string();
            let items: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "{}\"{}\": {}",
                        self.indent(indent + 1),
                        self.escape_string(k),
                        self.print_value(obj.get(*k).unwrap(), indent + 1)
                    )
                })
                .collect();
            result.push_str(&items.join(",\n"));
            result.push('\n');
            result.push_str(&self.indent(indent));
            result.push('}');
            result
        } else {
            let items: Vec<String> = keys
                .iter()
                .map(|k| {
                    format!(
                        "\"{}\":{}",
                        self.escape_string(k),
                        self.print_value(obj.get(*k).unwrap(), indent)
                    )
                })
                .collect();
            format!("{{{}}}", items.join(","))
        }
    }

    fn indent(&self, level: usize) -> String {
        "  ".repeat(level)
    }

    fn escape_string(&self, s: &str) -> String {
        s.chars()
            .flat_map(|c| match c {
                '"' => vec!['\\', '"'],
                '\\' => vec!['\\', '\\'],
                '\n' => vec!['\\', 'n'],
                '\r' => vec!['\\', 'r'],
                '\t' => vec!['\\', 't'],
                c if c.is_control() => {
                    // Unicode escape for control chars
                    format!("\\u{:04x}", c as u32).chars().collect()
                }
                c => vec![c],
            })
            .collect()
    }
}

// Convenience functions

/// Converts a Value to compact JSON string representation.
///
/// This function produces minified JSON output with no extra whitespace,
/// suitable for network transmission or storage where space is a concern.
///
/// # Examples
///
/// ```
/// use clove_lang::Value;
/// use clove_lang::output::to_json;
/// use std::collections::HashMap;
///
/// let mut obj = HashMap::new();
/// obj.insert("name".to_string(), Value::String("Alice".to_string()));
/// obj.insert("age".to_string(), Value::Integer(30));
///
/// let json = to_json(&Value::Object(obj));
/// // Output: {"age":30,"name":"Alice"}
/// ```
///
/// # Features
///
/// - No indentation or extra whitespace
/// - Deterministic output (object keys are sorted)
/// - Proper string escaping for special characters
/// - Integer and float values preserved accurately
pub fn to_json(value: &Value) -> String {
    JsonPrinter::new(false).print(value)
}

/// Converts a Value to pretty-printed JSON string representation.
///
/// This function produces human-readable JSON output with 2-space indentation,
/// suitable for debugging, logging, or user-facing output.
///
/// # Examples
///
/// ```
/// use clove_lang::Value;
/// use clove_lang::output::to_json_pretty;
/// use std::collections::HashMap;
///
/// let mut obj = HashMap::new();
/// obj.insert("name".to_string(), Value::String("Alice".to_string()));
/// obj.insert("age".to_string(), Value::Integer(30));
///
/// let json = to_json_pretty(&Value::Object(obj));
/// // Output:
/// // {
/// //   "age": 30,
/// //   "name": "Alice"
/// // }
/// ```
///
/// # Features
///
/// - 2-space indentation per level
/// - One element/property per line for arrays and objects
/// - Deterministic output (object keys are sorted)
/// - Proper string escaping for special characters
/// - Integer and float values preserved accurately
pub fn to_json_pretty(value: &Value) -> String {
    JsonPrinter::new(true).print(value)
}
