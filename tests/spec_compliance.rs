// Spec Compliance Tests
//
// Tests derived from REFERENCE.md to ensure implementation matches specification.
// Each test references the relevant section of the spec.

use clove_lang::{evaluator::Evaluator, lexer::Lexer, parser::Parser, value::Value};
use std::collections::HashMap;

fn eval_expr(expr_str: &str, doc: Value) -> Result<Value, String> {
    let lexer = Lexer::new(expr_str);
    let mut parser = Parser::new(lexer).map_err(|e| e.to_string())?;
    let expr = parser.parse().map_err(|e| e.to_string())?;

    let mut evaluator = Evaluator::new();
    evaluator.eval_expression(&expr, doc)
        .map_err(|e| e.to_string())
}

fn eval_query(query_str: &str, doc: Value) -> Result<Value, String> {
    let lexer = Lexer::new(query_str);
    let mut parser = Parser::new(lexer).map_err(|e| e.to_string())?;
    let query = parser.parse_query().map_err(|e| e.to_string())?;

    let mut evaluator = Evaluator::new();
    evaluator.eval_query(&query, doc)
        .map_err(|e| e.to_string())
}

fn json_object(pairs: Vec<(&str, Value)>) -> Value {
    let mut map = HashMap::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

// ============================================================================
// Section: Root Access (Syntax Reference)
// ============================================================================

#[test]
fn spec_root_access_entire_document() {
    // $ - The entire document
    let doc = Value::Integer(42);
    let result = eval_expr("$", doc).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn spec_root_access_field() {
    // $[field] - Access a field
    let doc = json_object(vec![("name", Value::String("Alice".into()))]);
    let result = eval_expr("$[name]", doc).unwrap();
    assert_eq!(result, Value::String("Alice".into()));
}

#[test]
fn spec_root_access_nested() {
    // $[field][nested] - Nested access
    let inner = json_object(vec![("city", Value::String("NYC".into()))]);
    let doc = json_object(vec![("address", inner)]);
    let result = eval_expr("$[address][city]", doc).unwrap();
    assert_eq!(result, Value::String("NYC".into()));
}

#[test]
fn spec_root_access_array_index() {
    // $[array][0] - Array index
    let arr = Value::Array(vec![Value::Integer(10), Value::Integer(20)]);
    let doc = json_object(vec![("items", arr)]);
    let result = eval_expr("$[items][0]", doc).unwrap();
    assert_eq!(result, Value::Integer(10));
}

// ============================================================================
// Section: Negative Array Indices
// ============================================================================

#[test]
#[ignore] // SPEC MISMATCH: negative indices not implemented
fn spec_negative_array_index_last() {
    // $[items][-1] - Last element
    let arr = Value::Array(vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)]);
    let doc = json_object(vec![("items", arr)]);
    let result = eval_expr("$[items][-1]", doc).unwrap();
    assert_eq!(result, Value::Integer(30));
}

#[test]
#[ignore] // SPEC MISMATCH: negative indices not implemented
fn spec_negative_array_index_second_to_last() {
    // $[items][-2] - Second-to-last element
    let arr = Value::Array(vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)]);
    let doc = json_object(vec![("items", arr)]);
    let result = eval_expr("$[items][-2]", doc).unwrap();
    assert_eq!(result, Value::Integer(20));
}

// ============================================================================
// Section: Numeric Key Behavior
// ============================================================================

#[test]
fn spec_integer_key_on_object() {
    // Integer keys on objects: Converted to string keys ("0", "42")
    let doc = json_object(vec![("0", Value::String("zero".into()))]);
    let result = eval_expr("$[0]", doc).unwrap();
    assert_eq!(result, Value::String("zero".into()));
}

#[test]
fn spec_float_key_on_object() {
    // Float keys on objects: Converted to string keys ("1.5", "3.14")
    let doc = json_object(vec![("1.5", Value::String("one-point-five".into()))]);
    let result = eval_expr("$[1.5]", doc).unwrap();
    assert_eq!(result, Value::String("one-point-five".into()));
}

// ============================================================================
// Section: Existence Check
// ============================================================================

#[test]
fn spec_existence_check_true_for_non_empty_array() {
    // $[array][?] - Check if array exists and is non-empty
    let arr = Value::Array(vec![Value::Integer(1)]);
    let doc = json_object(vec![("items", arr)]);
    let result = eval_expr("$[items][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_existence_check_false_for_empty_array() {
    let doc = json_object(vec![("items", Value::Array(vec![]))]);
    let result = eval_expr("$[items][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_existence_check_true_for_non_null_object() {
    let inner = json_object(vec![("x", Value::Integer(1))]);
    let doc = json_object(vec![("data", inner)]);
    let result = eval_expr("$[data][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_existence_check_false_for_null() {
    let doc = json_object(vec![("data", Value::Null)]);
    let result = eval_expr("$[data][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_existence_check_true_for_non_empty_string() {
    // For strings: true if exists and not empty
    let doc = json_object(vec![("text", Value::String("hello".into()))]);
    let result = eval_expr("$[text][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_existence_check_false_for_empty_string() {
    let doc = json_object(vec![("text", Value::String("".into()))]);
    let result = eval_expr("$[text][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// ============================================================================
// Section: Comparison Operators
// ============================================================================

#[test]
fn spec_operator_equal() {
    let doc = json_object(vec![("age", Value::Integer(25))]);
    let result = eval_expr("$[age] == 25", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_not_equal() {
    let doc = json_object(vec![("status", Value::String("ok".into()))]);
    let result = eval_expr("$[status] != \"error\"", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_less_than() {
    let doc = json_object(vec![("price", Value::Integer(50))]);
    let result = eval_expr("$[price] < 100", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_greater_than() {
    let doc = json_object(vec![("count", Value::Integer(15))]);
    let result = eval_expr("$[count] > 10", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_less_equal() {
    let doc = json_object(vec![("age", Value::Integer(65))]);
    let result = eval_expr("$[age] <= 65", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_greater_equal() {
    let doc = json_object(vec![("score", Value::Integer(90))]);
    let result = eval_expr("$[score] >= 90", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

// ============================================================================
// Section: Logical Operators
// ============================================================================

#[test]
fn spec_operator_and() {
    let doc = json_object(vec![
        ("age", Value::Integer(25)),
        ("verified", Value::Boolean(true)),
    ]);
    let result = eval_expr("$[age] > 18 and $[verified] == true", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_operator_or() {
    let doc = json_object(vec![("role", Value::String("admin".into()))]);
    let result = eval_expr("$[role] == \"admin\" or $[role] == \"mod\"", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

// ============================================================================
// Section: Arithmetic Operators
// ============================================================================

#[test]
fn spec_operator_addition() {
    let doc = json_object(vec![("price", Value::Integer(90))]);
    let result = eval_expr("$[price] + 10", doc).unwrap();
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn spec_operator_subtraction() {
    let doc = json_object(vec![
        ("total", Value::Integer(100)),
        ("tax", Value::Integer(10)),
    ]);
    let result = eval_expr("$[total] - $[tax]", doc).unwrap();
    assert_eq!(result, Value::Integer(90));
}

#[test]
fn spec_operator_multiplication() {
    // Per spec: mixed integer/float operations preserve integers when mathematically valid
    // 100 * 1.1 = 110.0, which is a whole number, so returns Integer
    let doc = json_object(vec![("price", Value::Integer(100))]);
    let result = eval_expr("$[price] * 1.1", doc).unwrap();
    assert_eq!(result, Value::Integer(110));
}

#[test]
fn spec_operator_division() {
    let doc = json_object(vec![("total", Value::Integer(100))]);
    let result = eval_expr("$[total] / 2", doc).unwrap();
    assert_eq!(result, Value::Integer(50));
}

#[test]
fn spec_operator_modulo() {
    let doc = json_object(vec![("count", Value::Integer(17))]);
    let result = eval_expr("$[count] % 10", doc).unwrap();
    assert_eq!(result, Value::Integer(7));
}

// ============================================================================
// Section: Arithmetic Type Behavior
// ============================================================================

#[test]
fn spec_arithmetic_int_int_returns_int() {
    // 100 + 10 → 110 (Integer)
    let doc = Value::Null;
    let result = eval_expr("100 + 10", doc).unwrap();
    assert_eq!(result, Value::Integer(110));
}

#[test]
fn spec_arithmetic_float_float_returns_float() {
    // 100.0 + 10.0 → 110.0 (Float)
    let doc = Value::Null;
    let result = eval_expr("100.0 + 10.0", doc).unwrap();
    assert_eq!(result, Value::Float(110.0));
}

#[test]
fn spec_arithmetic_mixed_whole_returns_int() {
    // 100.0 + 10 → 110 (Integer, result is whole)
    let doc = Value::Null;
    let result = eval_expr("100.0 + 10", doc).unwrap();
    assert_eq!(result, Value::Integer(110));
}

#[test]
fn spec_arithmetic_mixed_decimal_returns_float() {
    // 100.5 + 10 → 110.5 (Float, result has decimal)
    let doc = Value::Null;
    let result = eval_expr("100.5 + 10", doc).unwrap();
    assert_eq!(result, Value::Float(110.5));
}

#[test]
fn spec_arithmetic_exact_division_returns_int() {
    // 100 / 10 → 10 (Integer, exact division)
    let doc = Value::Null;
    let result = eval_expr("100 / 10", doc).unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn spec_arithmetic_inexact_division_returns_float() {
    // 100 / 3 → 33.333... (Float, inexact division)
    let doc = Value::Null;
    let result = eval_expr("100 / 3", doc).unwrap();
    match result {
        Value::Float(f) => assert!((f - 33.333333333333336).abs() < 0.0001),
        _ => panic!("Expected Float, got {:?}", result),
    }
}

// ============================================================================
// Section: String Operators
// ============================================================================

#[test]
fn spec_string_concatenation() {
    // $[first] + " " + $[last]
    let doc = json_object(vec![
        ("first", Value::String("John".into())),
        ("last", Value::String("Doe".into())),
    ]);
    let result = eval_expr("$[first] + \" \" + $[last]", doc).unwrap();
    assert_eq!(result, Value::String("John Doe".into()));
}

// ============================================================================
// Section: Array Methods
// ============================================================================

#[test]
fn spec_method_any_true() {
    // $[items].any(@[price] > 100)
    let items = Value::Array(vec![
        json_object(vec![("price", Value::Integer(50))]),
        json_object(vec![("price", Value::Integer(150))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].any(@[price] > 100)", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_any_false() {
    let items = Value::Array(vec![
        json_object(vec![("price", Value::Integer(50))]),
        json_object(vec![("price", Value::Integer(80))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].any(@[price] > 100)", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_method_any_simple_value() {
    // $[tags].any(@ == "urgent")
    let tags = Value::Array(vec![
        Value::String("normal".into()),
        Value::String("urgent".into()),
    ]);
    let doc = json_object(vec![("tags", tags)]);
    let result = eval_expr("$[tags].any(@ == \"urgent\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_all_true() {
    // $[items].all(@[status] == "shipped")
    let items = Value::Array(vec![
        json_object(vec![("status", Value::String("shipped".into()))]),
        json_object(vec![("status", Value::String("shipped".into()))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].all(@[status] == \"shipped\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_all_simple_value() {
    // $[scores].all(@ >= 60)
    let scores = Value::Array(vec![Value::Integer(70), Value::Integer(85), Value::Integer(65)]);
    let doc = json_object(vec![("scores", scores)]);
    let result = eval_expr("$[scores].all(@ >= 60)", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_filter() {
    // $[items].filter(@[category] == "electronics")
    let items = Value::Array(vec![
        json_object(vec![
            ("name", Value::String("Widget".into())),
            ("category", Value::String("hardware".into())),
        ]),
        json_object(vec![
            ("name", Value::String("Phone".into())),
            ("category", Value::String("electronics".into())),
        ]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].filter(@[category] == \"electronics\")", doc).unwrap();

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 1);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn spec_method_filter_simple_value() {
    // $[numbers].filter(@ > 0)
    let numbers = Value::Array(vec![
        Value::Integer(-1),
        Value::Integer(0),
        Value::Integer(5),
        Value::Integer(10),
    ]);
    let doc = json_object(vec![("numbers", numbers)]);
    let result = eval_expr("$[numbers].filter(@ > 0)", doc).unwrap();
    assert_eq!(result, Value::Array(vec![Value::Integer(5), Value::Integer(10)]));
}

#[test]
fn spec_method_map_extract() {
    // $[items].map(@[name])
    let items = Value::Array(vec![
        json_object(vec![("name", Value::String("Widget".into()))]),
        json_object(vec![("name", Value::String("Gadget".into()))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].map(@[name])", doc).unwrap();
    assert_eq!(result, Value::Array(vec![
        Value::String("Widget".into()),
        Value::String("Gadget".into()),
    ]));
}

#[test]
fn spec_method_map_transform() {
    // $[prices].map(@ * 1.1)
    // Per spec: mixed operations with whole results return integers
    let prices = Value::Array(vec![Value::Integer(100), Value::Integer(200)]);
    let doc = json_object(vec![("prices", prices)]);
    let result = eval_expr("$[prices].map(@ * 1.1)", doc).unwrap();
    assert_eq!(result, Value::Array(vec![Value::Integer(110), Value::Integer(220)]));
}

#[test]
fn spec_method_sum_simple() {
    // $[numbers].sum()
    let numbers = Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]);
    let doc = json_object(vec![("numbers", numbers)]);
    let result = eval_expr("$[numbers].sum()", doc).unwrap();
    assert_eq!(result, Value::Integer(6));
}

#[test]
fn spec_method_sum_with_lambda() {
    // $[items].sum(@[price])
    let items = Value::Array(vec![
        json_object(vec![("price", Value::Integer(50))]),
        json_object(vec![("price", Value::Integer(75))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].sum(@[price])", doc).unwrap();
    assert_eq!(result, Value::Integer(125));
}

#[test]
fn spec_method_count() {
    // $[items].count()
    let items = Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].count()", doc).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn spec_method_first() {
    // $[items].first()
    let items = Value::Array(vec![Value::Integer(10), Value::Integer(20)]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].first()", doc).unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn spec_method_first_empty() {
    // Returns null if empty
    let doc = json_object(vec![("items", Value::Array(vec![]))]);
    let result = eval_expr("$[items].first()", doc).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn spec_method_last() {
    // $[items].last()
    let items = Value::Array(vec![Value::Integer(10), Value::Integer(20)]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].last()", doc).unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn spec_method_last_empty() {
    // Returns null if empty
    let doc = json_object(vec![("items", Value::Array(vec![]))]);
    let result = eval_expr("$[items].last()", doc).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn spec_method_exists_non_empty() {
    // $[items].exists()
    let items = Value::Array(vec![Value::Integer(1)]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].exists()", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_exists_empty() {
    let doc = json_object(vec![("items", Value::Array(vec![]))]);
    let result = eval_expr("$[items].exists()", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_method_unique() {
    // $[tags].unique()
    let tags = Value::Array(vec![
        Value::String("a".into()),
        Value::String("b".into()),
        Value::String("a".into()),
        Value::String("c".into()),
    ]);
    let doc = json_object(vec![("tags", tags)]);
    let result = eval_expr("$[tags].unique()", doc).unwrap();
    assert_eq!(result, Value::Array(vec![
        Value::String("a".into()),
        Value::String("b".into()),
        Value::String("c".into()),
    ]));
}

#[test]
fn spec_method_sort_numbers() {
    // $[numbers].sort()
    let numbers = Value::Array(vec![Value::Integer(3), Value::Integer(1), Value::Integer(2)]);
    let doc = json_object(vec![("numbers", numbers)]);
    let result = eval_expr("$[numbers].sort()", doc).unwrap();
    assert_eq!(result, Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
}

#[test]
fn spec_method_sort_by_field() {
    // $[items].sort(@[price])
    let items = Value::Array(vec![
        json_object(vec![("price", Value::Integer(150))]),
        json_object(vec![("price", Value::Integer(50))]),
        json_object(vec![("price", Value::Integer(100))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_expr("$[items].sort(@[price])", doc).unwrap();

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            // First item should have price 50
            if let Value::Object(first) = &arr[0] {
                assert_eq!(first.get("price"), Some(&Value::Integer(50)));
            }
        }
        _ => panic!("Expected Array"),
    }
}

// ============================================================================
// Section: Type Method
// ============================================================================

#[test]
fn spec_method_type_object() {
    // Returns "object"
    let doc = json_object(vec![("x", Value::Integer(1))]);
    let result = eval_expr("$[x].type()", json_object(vec![("x", doc)])).unwrap();
    assert_eq!(result, Value::String("object".into()));
}

#[test]
fn spec_method_type_array() {
    let doc = json_object(vec![("x", Value::Array(vec![]))]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("array".into()));
}

#[test]
fn spec_method_type_string() {
    let doc = json_object(vec![("x", Value::String("hello".into()))]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("string".into()));
}

#[test]
fn spec_method_type_number_int() {
    // Both integer and float return "number"
    let doc = json_object(vec![("x", Value::Integer(42))]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("number".into()));
}

#[test]
fn spec_method_type_number_float() {
    let doc = json_object(vec![("x", Value::Float(3.14))]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("number".into()));
}

#[test]
fn spec_method_type_boolean() {
    let doc = json_object(vec![("x", Value::Boolean(true))]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("boolean".into()));
}

#[test]
fn spec_method_type_null() {
    let doc = json_object(vec![("x", Value::Null)]);
    let result = eval_expr("$[x].type()", doc).unwrap();
    assert_eq!(result, Value::String("null".into()));
}

// ============================================================================
// Section: String Methods
// ============================================================================

#[test]
fn spec_method_upper() {
    // $[name].upper()
    let doc = json_object(vec![("name", Value::String("alice".into()))]);
    let result = eval_expr("$[name].upper()", doc).unwrap();
    assert_eq!(result, Value::String("ALICE".into()));
}

#[test]
fn spec_method_lower() {
    // $[email].lower()
    let doc = json_object(vec![("email", Value::String("Alice@EXAMPLE.COM".into()))]);
    let result = eval_expr("$[email].lower()", doc).unwrap();
    assert_eq!(result, Value::String("alice@example.com".into()));
}

#[test]
fn spec_method_contains() {
    // $[text].contains("error")
    let doc = json_object(vec![("text", Value::String("An error occurred".into()))]);
    let result = eval_expr("$[text].contains(\"error\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_contains_false() {
    let doc = json_object(vec![("text", Value::String("All good".into()))]);
    let result = eval_expr("$[text].contains(\"error\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_method_startswith() {
    // $[url].startswith("https://")
    let doc = json_object(vec![("url", Value::String("https://example.com".into()))]);
    let result = eval_expr("$[url].startswith(\"https://\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_startswith_false() {
    let doc = json_object(vec![("url", Value::String("http://example.com".into()))]);
    let result = eval_expr("$[url].startswith(\"https://\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn spec_method_endswith() {
    // $[filename].endswith(".json")
    let doc = json_object(vec![("filename", Value::String("config.json".into()))]);
    let result = eval_expr("$[filename].endswith(\".json\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_method_endswith_false() {
    let doc = json_object(vec![("filename", Value::String("config.yaml".into()))]);
    let result = eval_expr("$[filename].endswith(\".json\")", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// ============================================================================
// Section: Filter Operator
// ============================================================================

#[test]
fn spec_filter_keeps_matching() {
    // ?($[status] == "active")
    let doc = json_object(vec![("status", Value::String("active".into()))]);
    let result = eval_query("$ | ?($[status] == \"active\")", doc).unwrap();
    match result {
        Value::Object(map) => assert_eq!(map.get("status"), Some(&Value::String("active".into()))),
        _ => panic!("Expected Object"),
    }
}

#[test]
fn spec_filter_rejects_non_matching() {
    // ?($[status] == "active") on inactive document returns null
    let doc = json_object(vec![("status", Value::String("inactive".into()))]);
    let result = eval_query("$ | ?($[status] == \"active\")", doc).unwrap();
    assert_eq!(result, Value::Null);
}

// ============================================================================
// Section: Transform Operator
// ============================================================================

#[test]
fn spec_transform_replace_field() {
    // ~($[price] := $[price] * 1.1) - Increase price by 10%
    // Per spec: mixed operations with whole results return integers
    let doc = json_object(vec![("price", Value::Integer(100))]);
    let result = eval_query("$ | ~($[price] := $[price] * 1.1)", doc).unwrap();
    match result {
        Value::Object(map) => assert_eq!(map.get("price"), Some(&Value::Integer(110))),
        _ => panic!("Expected Object"),
    }
}

#[test]
fn spec_transform_filter_array() {
    // ~($[items] := ?(@[status] == "ok")) - Filter items array
    let items = Value::Array(vec![
        json_object(vec![("status", Value::String("ok".into()))]),
        json_object(vec![("status", Value::String("error".into()))]),
        json_object(vec![("status", Value::String("ok".into()))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_query("$ | ~($[items] := ?(@[status] == \"ok\"))", doc).unwrap();

    match result {
        Value::Object(map) => {
            if let Some(Value::Array(arr)) = map.get("items") {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected items array");
            }
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn spec_transform_map_array() {
    // ~($[categories] := @[category]) - Map to categories
    let items = Value::Array(vec![
        json_object(vec![("category", Value::String("electronics".into()))]),
        json_object(vec![("category", Value::String("books".into()))]),
    ]);
    let doc = json_object(vec![("items", items)]);
    let result = eval_query("$ | ~($[items] := @[category])", doc).unwrap();

    match result {
        Value::Object(map) => {
            if let Some(Value::Array(arr)) = map.get("items") {
                assert_eq!(arr, &vec![
                    Value::String("electronics".into()),
                    Value::String("books".into()),
                ]);
            } else {
                panic!("Expected items array");
            }
        }
        _ => panic!("Expected Object"),
    }
}

// ============================================================================
// Section: Output Operator
// ============================================================================

#[test]
fn spec_output_entire_document() {
    // !($) - Return entire document
    let doc = json_object(vec![("x", Value::Integer(1))]);
    let result = eval_query("$ | !($)", doc.clone()).unwrap();
    assert_eq!(result, doc);
}

#[test]
fn spec_output_specific_field() {
    // !($[items]) - Return just items
    let items = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
    let doc = json_object(vec![("items", items.clone())]);
    let result = eval_query("$ | !($[items])", doc).unwrap();
    assert_eq!(result, items);
}

#[test]
fn spec_output_custom_object() {
    // !({"total": $[total], "count": $[count]}) - Return custom object
    let doc = json_object(vec![
        ("total", Value::Integer(100)),
        ("count", Value::Integer(5)),
    ]);
    let result = eval_query("$ | !({\"total\": $[total], \"count\": $[count]})", doc).unwrap();
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("total"), Some(&Value::Integer(100)));
            assert_eq!(map.get("count"), Some(&Value::Integer(5)));
        }
        _ => panic!("Expected Object"),
    }
}

// ============================================================================
// Section: Operator Precedence
// ============================================================================

#[test]
fn spec_precedence_multiplication_before_addition() {
    // 1 + 2 * 3 = 1 + 6 = 7
    let result = eval_expr("1 + 2 * 3", Value::Null).unwrap();
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn spec_precedence_parentheses_override() {
    // (1 + 2) * 3 = 3 * 3 = 9
    let result = eval_expr("(1 + 2) * 3", Value::Null).unwrap();
    assert_eq!(result, Value::Integer(9));
}

#[test]
fn spec_precedence_comparison_before_logical() {
    // $[x] > 5 and $[y] < 10 parses as ($[x] > 5) and ($[y] < 10)
    let doc = json_object(vec![
        ("x", Value::Integer(10)),
        ("y", Value::Integer(5)),
    ]);
    let result = eval_expr("$[x] > 5 and $[y] < 10", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn spec_precedence_and_before_or() {
    // true or false and false = true or (false and false) = true or false = true
    let result = eval_expr("true or false and false", Value::Null).unwrap();
    assert_eq!(result, Value::Boolean(true));
}
