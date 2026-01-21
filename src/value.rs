use std::collections::HashMap;

/// A JSON value used throughout the Clove query language.
///
/// This type represents all valid JSON types with a distinction between
/// integers and floats (unlike standard JSON which only has "number").
///
/// # Type Preservation
///
/// The language preserves the distinction between integers and floats:
/// - Arithmetic operations maintain integer types when results are whole
/// - Mixed operations intelligently preserve integers when mathematically valid
/// - High-precision decimal arithmetic prevents floating-point errors
///
/// # Examples
///
/// ```
/// use clove_lang::Value;
/// use std::collections::HashMap;
///
/// // Scalar values
/// let null = Value::Null;
/// let boolean = Value::Boolean(true);
/// let integer = Value::Integer(42);
/// let float = Value::Float(3.14);
/// let string = Value::String("hello".to_string());
///
/// // Collections
/// let array = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
///
/// let mut obj = HashMap::new();
/// obj.insert("key".to_string(), Value::String("value".to_string()));
/// let object = Value::Object(obj);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JSON null
    Null,

    /// JSON boolean (true/false)
    Boolean(bool),

    /// Floating-point number
    Float(f64),

    /// Integer number (preserved separately from floats)
    Integer(i64),

    /// UTF-8 string
    String(String),

    /// Array of values (homogeneous or heterogeneous)
    Array(Vec<Value>),

    /// Object with string keys and value values
    Object(HashMap<String, Value>),
}

impl Value {
    /// Check if the value is truthy (for conditions)
    pub fn is_truthy(&self) -> bool {
        use Value::*;
        match self {
            Null => false,
            Boolean(b) => *b,
            Float(n) => *n > 0.0,
            Integer(n) => *n > 0,
            String(s) => !s.is_empty(),
            Array(arr) => !arr.is_empty(),
            Object(obj) => !obj.is_empty(),
        }
    }

    /// Convert to boolean for conditions
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => self.is_truthy(),
        }
    }

    /// Get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Integer(n) => Some(*n as f64),
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            Value::Float(n) => Some(n.round() as i64),
            _ => None,
        }
    }

    /// Get as string (concatenation)
    pub fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Float(n) => n.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => format!("{:?}", self),
        }
    }
}
