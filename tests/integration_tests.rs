use clove_lang::{evaluator::Evaluator, lexer::Lexer, output::to_json_pretty, parser::Parser, value::Value};
use std::collections::HashMap;


fn eval_expr(expr_str: &str, doc: Value) -> Result<Value, String> {
    let lexer = Lexer::new(expr_str);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    let mut evaluator = Evaluator::new();
    evaluator.eval_expression(&expr, doc)
        .map_err(|e| format!("{:?}", e))
}

fn eval_query(query_str: &str, doc: Value) -> Result<Value, String> {
    let lexer = Lexer::new(query_str);
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    let mut evaluator = Evaluator::new();
    evaluator.eval_query(&query, doc)
        .map_err(|e| format!("{:?}", e))
}

fn json_object(pairs: Vec<(&str, Value)>) -> Value {
    let mut map = HashMap::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v);
    }
    Value::Object(map)
}

fn json_array(values: Vec<Value>) -> Value {
    Value::Array(values)
}

#[test]
fn test_simple_field_access() {
    let doc = json_object(vec![
        ("name", Value::String("John".into())),
        ("age", Value::Integer(30)),
    ]);
    
    let result = eval_expr("$[name]", doc).unwrap();
    assert_eq!(result, Value::String("John".into()));
}

#[test]
fn test_nested_access() {
    let doc = json_object(vec![
        ("user", json_object(vec![
            ("name", Value::String("Alice".into())),
            ("email", Value::String("alice@example.com".into())),
        ])),
    ]);
    
    let result = eval_expr("$[user][name]", doc).unwrap();
    assert_eq!(result, Value::String("Alice".into()));
}

#[test]
fn test_array_access() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            Value::String("first".into()),
            Value::String("second".into()),
            Value::String("third".into()),
        ])),
    ]);
    
    let result = eval_expr("$[items][1]", doc).unwrap();
    assert_eq!(result, Value::String("second".into()));
}

#[test]
fn test_arithmetic() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
    ]);
    
    let result = eval_expr("$[price] * 1.1", doc).unwrap();
    assert_eq!(result, Value::Integer(110));
}

#[test]
fn test_string_concatenation() {
    let doc = json_object(vec![
        ("first", Value::String("John".into())),
        ("last", Value::String("Doe".into())),
    ]);
    
    let result = eval_expr(r#"$[first] + " " + $[last]"#, doc).unwrap();
    assert_eq!(result, Value::String("John Doe".into()));
}

#[test]
fn test_comparison() {
    let doc = json_object(vec![
        ("age", Value::Integer(25)),
    ]);
    
    let result = eval_expr("$[age] > 18", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_logical_and() {
    let doc = json_object(vec![
        ("age", Value::Integer(25)),
        ("verified", Value::Boolean(true)),
    ]);
    
    let result = eval_expr("$[age] > 18 and $[verified] == true", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_filter_keeps_record() {
    let doc = json_object(vec![
        ("status", Value::String("active".into())),
        ("value", Value::Integer(42)),
    ]);
    
    let result = eval_query(r#"$ | ?($[status] == "active")"#, doc.clone()).unwrap();
    assert_eq!(result, doc); // Should return full document
}

#[test]
fn test_filter_rejects_record() {
    let doc = json_object(vec![
        ("status", Value::String("inactive".into())),
    ]);
    
    let result = eval_query(r#"$ | ?($[status] == "active")"#, doc).unwrap();
    assert_eq!(result, Value::Null); // Filtered out
}

#[test]
fn test_scope_reference() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            json_object(vec![("price", Value::Integer(100))]),
        ])),
    ]);
    
    let result = eval_query("$ | @items := $[items] | @items[0][price]", doc).unwrap();
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_existence_check_true() {
    let doc = json_object(vec![
        ("items", json_array(vec![Value::Integer(1)])),
    ]);
    
    let result = eval_expr("$[items][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_existence_check_false() {
    let doc = json_object(vec![
        ("items", json_array(vec![])),
    ]);
    
    let result = eval_expr("$[items][?]", doc).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_output_expression() {
    let doc = json_object(vec![
        ("name", Value::String("Alice".into())),
        ("age", Value::Integer(30)),
    ]);
    
    let result = eval_query("$ | !($[name])", doc).unwrap();
    assert_eq!(result, Value::String("Alice".into()));
}

#[test]
fn test_output_object_literal() {
    let doc = json_object(vec![
        ("name", Value::String("Bob".into())),
        ("age", Value::Integer(25)),
    ]);
    
    let result = eval_query(r#"$ | !({"user": $[name], "years": $[age]})"#, doc).unwrap();
    
    match result {
        Value::Object(obj) => {
            assert_eq!(obj.get("user"), Some(&Value::String("Bob".into())));
            assert_eq!(obj.get("years"), Some(&Value::Integer(25)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_env_var() {
    unsafe {
        std::env::set_var("TEST_VAR", "test_value");
    }
    
    let doc = json_object(vec![]);
    let result = eval_expr("$TEST_VAR", doc).unwrap();
    
    assert_eq!(result, Value::String("test_value".into()));
}

#[test]
fn test_complex_real_query() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            json_object(vec![
                ("name", Value::String("Widget".into())),
                ("price", Value::Integer(50)),
            ]),
            json_object(vec![
                ("name", Value::String("Gadget".into())),
                ("price", Value::Integer(150)),
            ]),
        ])),
        ("threshold", Value::Integer(100)),
    ]);
    
    // Check if any item exceeds threshold
    let result = eval_query(
        "$ | @threshold := $[threshold] | $[items][0][price] > @threshold",
        doc
    ).unwrap();
    
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_json_output() {
    let doc = json_object(vec![
        ("name", Value::String("Test".into())),
        ("count", Value::Integer(42)),
        ("items", json_array(vec![
            Value::Integer(1),
            Value::Integer(2),
        ])),
    ]);
    
    let result = eval_expr("$", doc).unwrap();
    let json_str = to_json_pretty(&result);
    
    println!("{}", json_str);
    // Outputs:
    // {
    //   "count": 42,
    //   "items": [
    //     1,
    //     2
    //   ],
    //   "name": "Test"
    // }
}

// ============================================================================
// Transform Tests
// ============================================================================

#[test]
fn test_transform_simple_replace() {
    let doc = json_object(vec![
        ("x", Value::Integer(5)),
        ("y", Value::Integer(10)),
    ]);
    
    let result = eval_query("$ | ~($[x] := 100)", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("x"), Some(&Value::Integer(100)));
            assert_eq!(map.get("y"), Some(&Value::Integer(10))); // Unchanged
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_replace_with_expression_mul() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("tax", Value::Integer(10)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] * 1.1)", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(110)));
        }
        _ => panic!("Expected object"),
    }
}


#[test]
fn test_transform_replace_with_expression_div() {
    let doc = json_object(vec![
        ("price", Value::Float(110.0)),
        ("tax", Value::Integer(10)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] / $[tax])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(11)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_replace_with_expression_substr() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("tax", Value::Float(10.0)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] - $[tax])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(90)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_replace_with_expression_add() {
    let doc = json_object(vec![
        ("price", Value::Float(100.0)),
        ("tax", Value::Integer(10)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] + $[tax])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(110)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_replace_with_expression_rem() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("tax", Value::Float(99.0)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] % $[tax])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(1)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_replace_with_expression_rem_2floats() {
    let doc = json_object(vec![
        ("price", Value::Float(100.0)),
        ("tax", Value::Float(99.0)),
    ]);
    
    let result = eval_query("$ | ~($[price] := $[price] % $[tax])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Float(1.0)));
        }
        _ => panic!("Expected object"),
    }
}


#[test]
fn test_transform_nested_field() {
    let doc = json_object(vec![
        ("user", json_object(vec![
            ("name", Value::String("Alice".into())),
            ("age", Value::Integer(30)),
        ])),
    ]);
    
    let result = eval_query(r#"$ | ~($[user][name] := "Bob")"#, doc).unwrap();
    
    match result {
        Value::Object(map) => {
            match map.get("user") {
                Some(Value::Object(user)) => {
                    assert_eq!(user.get("name"), Some(&Value::String("Bob".into())));
                    assert_eq!(user.get("age"), Some(&Value::Integer(30))); // Unchanged
                }
                _ => panic!("Expected nested object"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_array_element() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            Value::String("first".into()),
            Value::String("second".into()),
            Value::String("third".into()),
        ])),
    ]);
    
    let result = eval_query(r#"$ | ~($[items][1] := "UPDATED")"#, doc).unwrap();
    
    match &result {
        Value::Object(map) => {
            match map.get("items") {
                Some(Value::Array(arr)) => {
                    assert_eq!(arr.len(), 3);
                    assert_eq!(arr[0], Value::String("first".into()));
                    assert_eq!(arr[1], Value::String("UPDATED".into()), "{:?}", result);
                    assert_eq!(arr[2], Value::String("third".into()));
                }
                _ => panic!("Expected array"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_filter_array() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            json_object(vec![("price", Value::Integer(50))]),
            json_object(vec![("price", Value::Integer(150))]),
            json_object(vec![("price", Value::Integer(200))]),
        ])),
    ]);
    
    let result = eval_query("$ | ~($[items] := ?(@[price] > 100))", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            match map.get("items") {
                Some(Value::Array(arr)) => {
                    assert_eq!(arr.len(), 2); // Filtered to 2 items
                    
                    // Check first filtered item
                    match &arr[0] {
                        Value::Object(item) => {
                            assert_eq!(item.get("price"), Some(&Value::Integer(150)));
                        }
                        _ => panic!("Expected object"),
                    }
                }
                _ => panic!("Expected array"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_map_array_field() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            json_object(vec![
                ("name", Value::String("A".into())),
                ("price", Value::Integer(10)),
            ]),
            json_object(vec![
                ("name", Value::String("B".into())),
                ("price", Value::Integer(20)),
            ]),
        ])),
    ]);
    
    let result = eval_query("$ | ~($[items] := @[price])", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            match map.get("items") {
                Some(Value::Array(arr)) => {
                    assert_eq!(arr.len(), 2);
                    assert_eq!(arr[0], Value::Integer(10));
                    assert_eq!(arr[1], Value::Integer(20));
                }
                _ => panic!("Expected array"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_map_array_expression() {
    let doc = json_object(vec![
        ("items", json_array(vec![
            json_object(vec![("price", Value::Integer(100))]),
            json_object(vec![("price", Value::Integer(200))]),
        ])),
    ]);
    
    let result = eval_query("$ | ~($[items] := @[price] * 1.1)", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            match map.get("items") {
                Some(Value::Array(arr)) => {
                    assert_eq!(arr.len(), 2);
                    assert_eq!(arr[0], Value::Integer(110));
                    assert_eq!(arr[1], Value::Integer(220));
                }
                _ => panic!("Expected array"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_multiple_in_pipeline() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("discount", Value::Float(0.1)),
    ]);
    
    let result = eval_query(
        "$ | ~($[price] := $[price] * (1 - $[discount])) | ~($[final] := $[price])",
        doc
    ).unwrap();
    
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("price"), Some(&Value::Integer(90)));
            assert_eq!(map.get("final"), Some(&Value::Integer(90)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_transform_deeply_nested() {
    let doc = json_object(vec![
        ("a", json_object(vec![
            ("b", json_object(vec![
                ("c", Value::Integer(5)),
            ])),
        ])),
    ]);
    
    let result = eval_query("$ | ~($[a][b][c] := 100)", doc).unwrap();
    
    match result {
        Value::Object(map) => {
            match map.get("a") {
                Some(Value::Object(a)) => match a.get("b") {
                    Some(Value::Object(b)) => {
                        assert_eq!(b.get("c"), Some(&Value::Integer(100)));
                    }
                    _ => panic!("Expected nested object b"),
                }
                _ => panic!("Expected nested object a"),
            }
        }
        _ => panic!("Expected object"),
    }
}

// #[test]
// #[should_panic(expected = "Field 'nonexistent' not found")]
// fn test_transform_missing_field_error() {
//     let doc = json_object(vec![
//         ("x", Value::Integer(5)),
//     ]);
    
//     eval_query("$ | ~($[nonexistent] := 100)", doc).unwrap();
// }

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_transform_array_out_of_bounds() {
    let doc = json_object(vec![
        ("items", json_array(vec![Value::Integer(1)])),
    ]);
    
    eval_query("$ | ~($[items][10] := 100)", doc).unwrap();
}

#[test]
#[should_panic(expected = "requires array")]
fn test_transform_filter_on_non_array() {
    let doc = json_object(vec![
        ("value", Value::Integer(42)),
    ]);
    
    eval_query("$ | ~($[value] := ?(@[x] > 5))", doc).unwrap();
}

#[test]
fn test_debug_arithmetic_expression() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("discount", Value::Float(0.1)),
    ]);
    
    // Test just the arithmetic expression
    let result = eval_expr("$[price] * (1 - $[discount])", doc).unwrap();
    eprintln!("Arithmetic result: {:?}", result);
    assert_eq!(result, Value::Integer(90));
}

#[test]
fn test_debug_single_transform() {
    let doc = json_object(vec![
        ("price", Value::Integer(100)),
        ("discount", Value::Float(0.1)),
    ]);
    
    // Test single transform
    let result = eval_query("$ | ~($[price] := $[price] * (1 - $[discount]))", doc).unwrap();
    eprintln!("Transform result: {:?}", result);
    
    match result {
        Value::Object(ref map) => {
            eprintln!("price value: {:?}", map.get("price"));
            assert_eq!(map.get("price"), Some(&Value::Integer(90)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_debug_simple_subtract() {
    let doc = json_object(vec![
        ("discount", Value::Float(0.1)),
    ]);
    
    // Test 1 - $[discount]
    let result = eval_expr("1 - $[discount]", doc).unwrap();
    eprintln!("1 - $[discount] = {:?}", result);
}
