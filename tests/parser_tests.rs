// tests/parser_tests.rs

use clove_lang::lexer::Lexer;
use clove_lang::parser::Parser;
use clove_lang::ast::{BinOp, Expr, Statement};

// ============================================================================
// Simple tests
// ============================================================================

#[test]
fn test_comparison() {
    let lexer = Lexer::new("$[price] > 100");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();

    assert!(matches!(
        expr,
        Expr::BinaryOp {
            op: BinOp::GreaterThan,
            ..
        }
    ));
}

#[test]
fn test_parentheses() {
    let lexer = Lexer::new("(1 + 2) * 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();

    // Should be: Multiply(Add(1, 2), 3)
    match expr {
        Expr::BinaryOp {
            op: BinOp::Multiply,
            left,
            right,
        } => {
            match *left {
                Expr::BinaryOp { op: BinOp::Add, .. } => {} // Good!
                _ => panic!("Expected addition in left"),
            }
            assert!(matches!(*right, Expr::Integer(n) if n == 3));
        }
        _ => panic!("Expected multiplication"),
    }
}

#[test]
fn test_arithmetic() {
    let lexer = Lexer::new("1 + 2 * 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();

    // Should be: Add(1, Multiply(2, 3))
    match expr {
        Expr::BinaryOp {
            op: BinOp::Add,
            left,
            right,
        } => {
            assert!(matches!(*left, Expr::Integer(n) if n == 1));
            match *right {
                Expr::BinaryOp {
                    op: BinOp::Multiply,
                    left,
                    right,
                } => {
                    assert!(matches!(*left, Expr::Integer(n) if n == 2));
                    assert!(matches!(*right, Expr::Integer(n) if n == 3));
                }
                _ => panic!("Expected multiplication"),
            }
        }
        _ => panic!("Expected addition"),
    }
}


// ============================================================================
// Literals and Primitives
// ============================================================================

#[test]
fn test_parse_number() {
    let lexer = Lexer::new("42");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Integer(n) if n == 42));
}

#[test]
fn test_parse_float() {
    let lexer = Lexer::new("3.15");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Float(n) if (n - 3.15).abs() < 0.001));
}

#[test]
fn test_parse_string() {
    let lexer = Lexer::new(r##""hello world""##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::String(s) if s == "hello world"));
}

#[test]
fn test_parse_boolean_true() {
    let lexer = Lexer::new("true");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Boolean(true)));
}

#[test]
fn test_parse_boolean_false() {
    let lexer = Lexer::new("false");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Boolean(false)));
}

#[test]
fn test_parse_null() {
    let lexer = Lexer::new("null");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Null));
}

// ============================================================================
// Object Literals
// ============================================================================

#[test]
fn test_parse_empty_object() {
    let lexer = Lexer::new("{}");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 0);
        }
        _ => panic!("Expected Object, got {:?}", expr),
    }
}

#[test]
fn test_parse_object_single_field() {
    let lexer = Lexer::new(r##"{"name": "John"}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "name");
            assert!(matches!(pairs[0].1, Expr::String(ref s) if s == "John"));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_object_multiple_fields() {
    let lexer = Lexer::new(r##"{"name": "John", "age": 30, "active": true}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 3);
            assert_eq!(pairs[0].0, "name");
            assert_eq!(pairs[1].0, "age");
            assert_eq!(pairs[2].0, "active");
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_object_with_identifier_keys() {
    let lexer = Lexer::new("{name: 42, age: 30}");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0, "name");
            assert_eq!(pairs[1].0, "age");
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_object_with_expressions() {
    let lexer = Lexer::new(r##"{"total": $[price] * 1.1, "name": "Item"}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Expr::BinaryOp { .. }));
            assert!(matches!(pairs[1].1, Expr::String(_)));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_nested_objects() {
    let lexer = Lexer::new(r##"{"user": {"name": "John", "age": 30}}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 1);
            assert!(matches!(pairs[0].1, Expr::Object(_)));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_object_trailing_comma() {
    // Should handle trailing comma gracefully (or reject it)
    let lexer = Lexer::new(r##"{"name": "John",}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // This will depend on your implementation
    // Either it parses successfully or panics
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 1);
        }
        _ => panic!("Expected Object"),
    }
}

// ============================================================================
// Array Literals
// ============================================================================

#[test]
fn test_parse_empty_array() {
    let lexer = Lexer::new("[]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 0);
        }
        _ => panic!("Expected Array, got {:?}", expr),
    }
}

#[test]
fn test_parse_array_numbers() {
    let lexer = Lexer::new("[1, 2, 3]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[0], Expr::Integer(n) if n == 1));
            assert!(matches!(elements[1], Expr::Integer(n) if n == 2));
            assert!(matches!(elements[2], Expr::Integer(n) if n == 3));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_array_strings() {
    let lexer = Lexer::new(r##"["a", "b", "c"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[0], Expr::String(ref s) if s == "a"));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_array_mixed_types() {
    let lexer = Lexer::new(r##"[1, 1.0, "hello", true, null]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 5);
            assert!(matches!(elements[0], Expr::Integer(_)));
            assert!(matches!(elements[1], Expr::Float(_)));
            assert!(matches!(elements[2], Expr::String(_)));
            assert!(matches!(elements[3], Expr::Boolean(true)));
            assert!(matches!(elements[4], Expr::Null));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_array_with_expressions() {
    let lexer = Lexer::new("[$[price] * 1.1, $[quantity]]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::BinaryOp { .. }));
            assert!(matches!(elements[1], Expr::Access { .. }));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_nested_arrays() {
    let lexer = Lexer::new("[[1, 2], [3, 4]]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::Array(_)));
            assert!(matches!(elements[1], Expr::Array(_)));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_array_vs_access() {
    // [1, 2] is array literal
    let lexer = Lexer::new("[1, 2]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Array(_)));
    
    // $[items] is access
    let lexer = Lexer::new("$[items]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Access { .. }));
}


// ============================================================================
// Root and References
// ============================================================================

#[test]
fn test_parse_root() {
    let lexer = Lexer::new("$");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::Root));
}

#[test]
fn test_parse_env_var() {
    let lexer = Lexer::new("$HOME");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::EnvVar(name) if name == "HOME"));
}

#[test]
fn test_parse_lambda_param() {
    let lexer = Lexer::new("@");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::LambdaParam));
}

#[test]
fn test_parse_scope_ref() {
    let lexer = Lexer::new("@items");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::ScopeRef(name) if name == "items"));
}

#[test]
fn test_parse_arg_ref() {
    let lexer = Lexer::new("@1");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::ArgRef(1)));
}

#[test]
fn test_parse_arg_ref_multi_digit() {
    let lexer = Lexer::new("@10");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    assert!(matches!(expr, Expr::ArgRef(10)));
}

// ============================================================================
// Basic Access Patterns
// ============================================================================

#[test]
fn test_parse_simple_bracket_access() {
    let lexer = Lexer::new("$[field]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            assert!(matches!(*key, Expr::Key(s) if s == "field"));
        }
        _ => panic!("Expected Access, got {:?}", expr),
    }
}

#[test]
fn test_parse_nested_bracket_access() {
    let lexer = Lexer::new("$[items][name]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "name"));
            match *object {
                Expr::Access { object: inner_obj, key: inner_key } => {
                    assert!(matches!(*inner_obj, Expr::Root));
                    assert!(matches!(*inner_key, Expr::Key(s) if s == "items"));
                }
                _ => panic!("Expected nested Access"),
            }
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_array_index() {
    let lexer = Lexer::new("$[items][0]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object: _, key } => {
            assert!(matches!(*key, Expr::Integer (n) if n == 0));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_dot_notation() {
    let lexer = Lexer::new("$.field");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            assert!(matches!(*key, Expr::Key(s) if s == "field"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_mixed_access() {
    let lexer = Lexer::new("$[items].name[0]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Verify it parses without panicking
    assert!(matches!(expr, Expr::Access { .. }));
}

#[test]
fn test_parse_deeply_nested_access() {
    let lexer = Lexer::new("$[user][profile][settings][theme]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::Access { .. }));
}

// ============================================================================
// Special Characters in Keys
// ============================================================================

#[test]
fn test_parse_quoted_key_with_hyphen() {
    let lexer = Lexer::new(r##"$["first-name"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            assert!(matches!(*key, Expr::Key(s) if s == "first-name"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_quoted_key_with_at() {
    let lexer = Lexer::new(r##"$["@timestamp"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object: _, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "@timestamp"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_quoted_key_with_dot() {
    let lexer = Lexer::new(r##"$["user.email"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object: _, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "user.email"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_quoted_key_with_space() {
    let lexer = Lexer::new(r##"$["field with spaces"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object: _, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "field with spaces"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_quoted_key_with_dollar() {
    let lexer = Lexer::new(r##"$["$ref"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object: _, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "$ref"));
        }
        _ => panic!("Expected Access"),
    }
}

// ============================================================================
// Computed Keys (Expressions in Brackets)
// ============================================================================

#[test]
fn test_parse_computed_key_simple() {
    let lexer = Lexer::new("$[0 + 1]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            // Key should be BinaryOp
            assert!(matches!(*key, Expr::BinaryOp { op: BinOp::Add, .. }));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_nested_access_as_key() {
    let lexer = Lexer::new("$[$[key_field]]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            // Key should be another Access
            assert!(matches!(*key, Expr::Access { .. }));
        }
        _ => panic!("Expected Access"),
    }
}

// ============================================================================
// Existence Check
// ============================================================================

#[test]
fn test_parse_existence_check() {
    let lexer = Lexer::new("$[items][?]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::ExistenceCheck(inner) => {
            match *inner {
                Expr::Access { object, key } => {
                    assert!(matches!(*object, Expr::Root));
                    assert!(matches!(*key, Expr::Key(s) if s == "items"));
                }
                _ => panic!("Expected Access inside ExistenceCheck"),
            }
        }
        _ => panic!("Expected ExistenceCheck, got {:?}", expr),
    }
}

#[test]
fn test_parse_nested_existence_check() {
    let lexer = Lexer::new("$[items][0][?]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::ExistenceCheck(inner) => {
            // Inner should be Access { Access { Root, "items" }, 0 }
            assert!(matches!(*inner, Expr::Access { .. }));
        }
        _ => panic!("Expected ExistenceCheck"),
    }
}

#[test]
fn test_parse_question_mark_as_key() {
    // Quoted question mark is a literal key
    let lexer = Lexer::new(r##"$["?"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::Root));
            assert!(matches!(*key, Expr::Key(s) if s == "?"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
#[should_panic(expected = "Expected Eof, got ")]
fn test_cannot_access_after_existence_check() {
    // This should parse fine but fail at evaluation
    let lexer = Lexer::new("$[items][?][0]");
    let mut parser = Parser::new(lexer);
    let _expr = parser.parse();
    
    // The parse should succeed but create:
    // Access { ExistenceCheck(...), Number(0) }
    // Which will fail at evaluation
}



// ============================================================================
// Arithmetic Operators
// ============================================================================

#[test]
fn test_parse_addition() {
    let lexer = Lexer::new("1 + 2");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::Add, left, right } => {
            assert!(matches!(*left, Expr::Integer (n) if n == 1));
            assert!(matches!(*right, Expr::Integer (n) if n == 2));
        }
        _ => panic!("Expected Add"),
    }
}

#[test]
fn test_parse_subtraction() {
    let lexer = Lexer::new("10 - 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Subtract, .. }));
}

#[test]
fn test_parse_multiplication() {
    let lexer = Lexer::new("3 * 4");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Multiply, .. }));
}

#[test]
fn test_parse_division() {
    let lexer = Lexer::new("8 / 2");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Divide, .. }));
}

#[test]
fn test_parse_modulo() {
    let lexer = Lexer::new("10 % 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Modulo, .. }));
}

#[test]
fn test_parse_arithmetic_precedence() {
    let lexer = Lexer::new("1 + 2 * 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Add(1, Multiply(2, 3))
    match expr {
        Expr::BinaryOp { op: BinOp::Add, left, right } => {
            assert!(matches!(*left, Expr::Integer (n) if n == 1));
            match *right {
                Expr::BinaryOp { op: BinOp::Multiply, left, right } => {
                    assert!(matches!(*left, Expr::Integer (n) if n == 2));
                    assert!(matches!(*right, Expr::Integer (n) if n == 3));
                }
                _ => panic!("Expected Multiply"),
            }
        }
        _ => panic!("Expected Add"),
    }
}

#[test]
fn test_parse_left_associativity() {
    let lexer = Lexer::new("1 - 2 - 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Subtract(Subtract(1, 2), 3) = (1 - 2) - 3
    match expr {
        Expr::BinaryOp { op: BinOp::Subtract, left, right } => {
            assert!(matches!(*right, Expr::Integer (n) if n == 3));
            match *left {
                Expr::BinaryOp { op: BinOp::Subtract, left, right } => {
                    assert!(matches!(*left, Expr::Integer (n) if n == 1));
                    assert!(matches!(*right, Expr::Integer (n) if n == 2));
                }
                _ => panic!("Expected nested Subtract"),
            }
        }
        _ => panic!("Expected Subtract"),
    }
}

#[test]
fn test_parse_string_concatenation() {
    let lexer = Lexer::new(r##""hello" + " " + "world""##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should use Add operator for string concatenation
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Add, .. }));
}

#[test]
fn test_parse_field_arithmetic() {
    let lexer = Lexer::new("$[price] * 1.1");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::Multiply, left, right } => {
            assert!(matches!(*left, Expr::Access { .. }));
            assert!(matches!(*right, Expr::Float(n) if (n - 1.1).abs() < 0.001));
        }
        _ => panic!("Expected Multiply"),
    }
}

// ============================================================================
// Parentheses and Precedence
// ============================================================================

#[test]
fn test_parse_parentheses_simple() {
    let lexer = Lexer::new("(5)");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::Integer (n) if n == 5));
}

#[test]
fn test_parse_parentheses_override_precedence() {
    let lexer = Lexer::new("(1 + 2) * 3");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Multiply(Add(1, 2), 3)
    match expr {
        Expr::BinaryOp { op: BinOp::Multiply, left, right } => {
            match *left {
                Expr::BinaryOp { op: BinOp::Add, left, right } => {
                    assert!(matches!(*left, Expr::Integer (n) if n == 1));
                    assert!(matches!(*right, Expr::Integer (n) if n == 2));
                }
                _ => panic!("Expected Add in left"),
            }
            assert!(matches!(*right, Expr::Integer (n) if n == 3));
        }
        _ => panic!("Expected Multiply"),
    }
}

#[test]
fn test_parse_nested_parentheses() {
    let lexer = Lexer::new("((1 + 2))");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Add, .. }));
}

#[test]
fn test_parse_complex_precedence() {
    let lexer = Lexer::new("1 + 2 * 3 + 4");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Add(Add(1, Multiply(2, 3)), 4)
    match expr {
        Expr::BinaryOp { op: BinOp::Add, left, right } => {
            assert!(matches!(*right, Expr::Integer (n) if n == 4));
            match *left {
                Expr::BinaryOp { op: BinOp::Add, left, right } => {
                    assert!(matches!(*left, Expr::Integer (n) if n == 1));
                    assert!(matches!(*right, Expr::BinaryOp { op: BinOp::Multiply, .. }));
                }
                _ => panic!("Expected nested Add"),
            }
        }
        _ => panic!("Expected Add"),
    }
}

// ============================================================================
// Comparison Operators
// ============================================================================

#[test]
fn test_parse_equal() {
    let lexer = Lexer::new("$[x] == 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Equal, .. }));
}

#[test]
fn test_parse_not_equal() {
    let lexer = Lexer::new("$[x] != 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::NotEqual, .. }));
}

#[test]
fn test_parse_less_than() {
    let lexer = Lexer::new("$[x] < 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::LessThan, .. }));
}

#[test]
fn test_parse_greater_than() {
    let lexer = Lexer::new("$[x] > 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::GreaterThan, .. }));
}

#[test]
fn test_parse_less_equal() {
    let lexer = Lexer::new("$[x] <= 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::LessEqual, .. }));
}

#[test]
fn test_parse_greater_equal() {
    let lexer = Lexer::new("$[x] >= 5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::GreaterEqual, .. }));
}

#[test]
fn test_parse_comparison_with_arithmetic() {
    let lexer = Lexer::new("$[price] * 1.1 > 100");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: GreaterThan(Multiply(price, 1.1), 100)
    match expr {
        Expr::BinaryOp { op: BinOp::GreaterThan, left, right } => {
            assert!(matches!(*left, Expr::BinaryOp { op: BinOp::Multiply, .. }));
            assert!(matches!(*right, Expr::Integer (n) if n == 100));
        }
        _ => panic!("Expected GreaterThan"),
    }
}

// ============================================================================
// Logical Operators
// ============================================================================

#[test]
fn test_parse_and() {
    let lexer = Lexer::new("$[x] > 5 and $[y] < 10");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::And, .. }));
}

#[test]
fn test_parse_or() {
    let lexer = Lexer::new("$[x] == 1 or $[x] == 2");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Or, .. }));
}

#[test]
fn test_parse_and_or_precedence() {
    let lexer = Lexer::new("$[a] and $[b] or $[c]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Or(And(a, b), c)
    match expr {
        Expr::BinaryOp { op: BinOp::Or, left, right } => {
            assert!(matches!(*left, Expr::BinaryOp { op: BinOp::And, .. }));
            assert!(matches!(*right, Expr::Access { .. }));
        }
        _ => panic!("Expected Or"),
    }
}

#[test]
fn test_parse_complex_logical() {
    let lexer = Lexer::new("($[a] > 5 and $[b] < 10) or $[c] == 0");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Or, .. }));
}

#[test]
fn test_parse_multiple_and() {
    let lexer = Lexer::new("$[a] and $[b] and $[c]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be left-associative: And(And(a, b), c)
    match expr {
        Expr::BinaryOp { op: BinOp::And, left, right } => {
            assert!(matches!(*left, Expr::BinaryOp { op: BinOp::And, .. }));
            assert!(matches!(*right, Expr::Access { .. }));
        }
        _ => panic!("Expected And"),
    }
}

#[test]
fn test_parse_and_simple() {
    let lexer = Lexer::new("true and false");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::And, left, right } => {
            assert!(matches!(*left, Expr::Boolean(true)));
            assert!(matches!(*right, Expr::Boolean(false)));
        }
        _ => panic!("Expected And, got {:?}", expr),
    }
}

#[test]
fn test_parse_and_with_comparisons() {
    let lexer = Lexer::new("$[x] > 5 and $[y] < 10");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::And, left, right } => {
            assert!(matches!(*left, Expr::BinaryOp { op: BinOp::GreaterThan, .. }));
            assert!(matches!(*right, Expr::BinaryOp { op: BinOp::LessThan, .. }));
        }
        _ => panic!("Expected And"),
    }
}

#[test]
fn test_parse_multiple_and_chains() {
    let lexer = Lexer::new("$[a] and $[b] and $[c] and $[d]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: And(And(And(a, b), c), d)
    match expr {
        Expr::BinaryOp { op: BinOp::And, .. } => {
            // Successfully parsed chained ANDs
        }
        _ => panic!("Expected And"),
    }
}

#[test]
fn test_parse_and_with_parentheses() {
    let lexer = Lexer::new("($[a] and $[b]) and ($[c] and $[d])");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::And, .. }));
}


// ============================================================================
// Unary Minus
// ============================================================================

#[test]
fn test_parse_unary_minus() {
    let lexer = Lexer::new("-5");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Represented as 0 - 5
    match expr {
        Expr::BinaryOp { op: BinOp::Subtract, left, right } => {
            assert!(matches!(*left, Expr::Integer (n) if n == 0));
            assert!(matches!(*right, Expr::Integer (n) if n == 5));
        }
        _ => panic!("Expected Subtract for unary minus"),
    }
}

#[test]
fn test_parse_unary_minus_with_expression() {
    let lexer = Lexer::new("-(5 + 3)");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::Subtract, left, right } => {
            assert!(matches!(*left, Expr::Integer (n) if n == 0));
            assert!(matches!(*right, Expr::BinaryOp { op: BinOp::Add, .. }));
        }
        _ => panic!("Expected Subtract"),
    }
}

// ============================================================================
// Complex Expressions
// ============================================================================

#[test]
fn test_parse_real_world_query() {
    let lexer = Lexer::new(r##"$[items][0][price] * 1.1 + $[tax]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Just verify it parses
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Add, .. }));
}

#[test]
fn test_parse_complex_condition() {
    let lexer = Lexer::new(
        r##"($[age] >= 18 and $[age] <= 65) or $[status] == "exempt""##
    );
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Or, .. }));
}

#[test]
fn test_parse_nested_access_with_arithmetic() {
    let lexer = Lexer::new("$[items][$[current] + 1][price]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::Access { .. }));
}

#[test]
fn test_parse_scope_ref_with_access() {
    let lexer = Lexer::new("@items[0][price]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    // Should be: Access(Access(ScopeRef("items"), 0), "price")
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*key, Expr::Key(s) if s == "price"));
            match *object {
                Expr::Access { object, key } => {
                    assert!(matches!(*key, Expr::Integer (n) if n == 0));
                    assert!(matches!(*object, Expr::ScopeRef(s) if s == "items"));
                }
                _ => panic!("Expected nested Access"),
            }
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_lambda_param_with_access() {
    let lexer = Lexer::new("@[price]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::LambdaParam));
            assert!(matches!(*key, Expr::Key(s) if s == "price"));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_arg_ref_with_access() {
    let lexer = Lexer::new("@1[price]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { object, key } => {
            assert!(matches!(*object, Expr::ArgRef(1)));
            assert!(matches!(*key, Expr::Key(s) if s == "price"));
        }
        _ => panic!("Expected Access"),
    }
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_parse_output_with_object() {
    // This is for later when you add output parsing, but the object part works now
    let lexer = Lexer::new(r##"{"total": $[items][0][price] * 1.1, "count": 5}"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Object(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert!(matches!(pairs[0].1, Expr::BinaryOp { .. }));
            assert!(matches!(pairs[1].1, Expr::Integer(_)));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_parse_array_of_objects() {
    let lexer = Lexer::new(r##"[{"name": "a"}, {"name": "b"}]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Array(elements) => {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], Expr::Object(_)));
            assert!(matches!(elements[1], Expr::Object(_)));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_parse_complex_nested_structure() {
    let lexer = Lexer::new(
        r##"{"items": [1, 2, 3], "metadata": {"count": 3, "total": 6}}"##
    );
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::Object(_)));
}

// ============================================================================
// Error Cases (Should Panic)
// ============================================================================

#[test]
#[should_panic(expected = "Unexpected use of identifiers - identifiers must be a part of access expressions")]
fn test_parse_bare_identifier_error() {
    let lexer = Lexer::new("items");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_unclosed_bracket() {
    let lexer = Lexer::new("$[items");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_unclosed_parenthesis() {
    let lexer = Lexer::new("(1 + 2");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic]
fn test_parse_empty_brackets() {
    let lexer = Lexer::new("$[]");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected identifier after '.'")]
fn test_parse_dot_without_identifier() {
    let lexer = Lexer::new("$.5");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected string or identifier as object key")]
fn test_parse_object_invalid_key() {
    let lexer = Lexer::new("{123: value}");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_object_missing_colon() {
    let lexer = Lexer::new(r##"{"name" "value"}"##);
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Unexpected token in primary expression")]
fn test_parse_object_missing_value() {
    let lexer = Lexer::new(r##"{"name":}"##);
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_object_unclosed() {
    let lexer = Lexer::new(r##"{"name": "value""##);
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_array_unclosed() {
    let lexer = Lexer::new("[1, 2, 3");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Unexpected token")]
fn test_parse_invalid_primary() {
    let lexer = Lexer::new("?");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected Eof")]
fn test_parse_pipe_in_expression() {
    let lexer = Lexer::new("5 | 3");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_mismatched_parentheses() {
    let lexer = Lexer::new("(1 + 2))");
    let mut parser = Parser::new(lexer);
    parser.parse();
}

#[test]
#[should_panic(expected = "Expected identifier after '.'")]
fn test_parse_dot_followed_by_number() {
    let lexer = Lexer::new("$.123");
    let mut parser = Parser::new(lexer);
    parser.parse();
}


// ============================================================================
// The "Unreachable" Case
// ============================================================================

#[test]
fn test_parse_access_key_identifier() {
    // This should go through the Identifier branch
    let lexer = Lexer::new("$[field]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { key, .. } => {
            assert!(matches!(*key, Expr::Key(_)));
        }
        _ => panic!("Expected Access"),
    }
}

#[test]
fn test_parse_access_key_string() {
    // This should go through the String branch
    let lexer = Lexer::new(r##"$["field"]"##);
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::Access { key, .. } => {
            assert!(matches!(*key, Expr::Key(_)));
        }
        _ => panic!("Expected Access"),
    }
}

// ============================================================================
// Edge Cases for Complete Coverage
// ============================================================================

#[test]
fn test_parse_or_simple() {
    let lexer = Lexer::new("true or false");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    match expr {
        Expr::BinaryOp { op: BinOp::Or, left, right } => {
            assert!(matches!(*left, Expr::Boolean(true)));
            assert!(matches!(*right, Expr::Boolean(false)));
        }
        _ => panic!("Expected Or"),
    }
}

#[test]
fn test_parse_multiple_or_chains() {
    let lexer = Lexer::new("$[a] or $[b] or $[c]");
    let mut parser = Parser::new(lexer);
    let expr = parser.parse();
    
    assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Or, .. }));
}

#[test]
fn test_parse_all_comparison_operators() {
    let operators = vec![
        ("==", BinOp::Equal),
        ("!=", BinOp::NotEqual),
        ("<", BinOp::LessThan),
        (">", BinOp::GreaterThan),
        ("<=", BinOp::LessEqual),
        (">=", BinOp::GreaterEqual),
    ];
    
    for (op_str, expected_op) in operators {
        let input = format!("5 {} 3", op_str);
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let expr = parser.parse();
        
        match expr {
            Expr::BinaryOp { op, .. } => {
                assert_eq!(op, expected_op, "Failed for operator {}", op_str);
            }
            _ => panic!("Expected comparison for {}", op_str),
        }
    }
}

#[test]
fn test_parse_all_arithmetic_operators() {
    let operators = vec![
        ("+", BinOp::Add),
        ("-", BinOp::Subtract),
        ("*", BinOp::Multiply),
        ("/", BinOp::Divide),
        ("%", BinOp::Modulo),
    ];
    
    for (op_str, expected_op) in operators {
        let input = format!("10 {} 2", op_str);
        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let expr = parser.parse();
        
        match expr {
            Expr::BinaryOp { op, .. } => {
                assert_eq!(op, expected_op, "Failed for operator {}", op_str);
            }
            _ => panic!("Expected arithmetic for {}", op_str),
        }
    }
}


// ============================================================================
// Filter Statements
// ============================================================================

#[test]
fn test_parse_simple_filter() {
    let lexer = Lexer::new("$ | ?($[status] == \"active\")");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    match &query.statements[0] {
        Statement::Filter(expr) => {
            assert!(matches!(expr, Expr::BinaryOp { op: BinOp::Equal, .. }));
        }
        _ => panic!("Expected Filter statement"),
    }
}

#[test]
fn test_parse_complex_filter() {
    let lexer = Lexer::new("$ | ?(($[age] > 18 and $[verified] == true) or $[admin] == true)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    assert!(matches!(query.statements[0], Statement::Filter(_)));
}

// ============================================================================
// Transform Statements
// ============================================================================

#[test]
fn test_parse_simple_transform() {
    let lexer = Lexer::new("$ | ~($[price] := $[price] * 1.1)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    match &query.statements[0] {
        Statement::Transform { target, value } => {
            assert!(matches!(target, Expr::Access { .. }));
            assert!(matches!(value, Expr::BinaryOp { .. }));
        }
        _ => panic!("Expected Transform statement"),
    }
}

#[test]
fn test_parse_array_filter_transform() {
    let lexer = Lexer::new("$ | ~($[items] := ?(@[price] > 100))");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    match &query.statements[0] {
        Statement::Transform { target, value } => {
            assert!(matches!(target, Expr::Access { .. }));
            // Value should be a Filter expression
            assert!(matches!(value, Expr::Filter(_)));
        }
        _ => panic!("Expected Transform statement"),
    }
}

// ============================================================================
// Scope Definitions
// ============================================================================

#[test]
fn test_parse_scope_definition() {
    let lexer = Lexer::new("$ | @items := $[items]");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    match &query.statements[0] {
        Statement::ScopeDefinition { name, path } => {
            assert_eq!(name, "items");
            assert!(matches!(path, Expr::Access { .. }));
        }
        _ => panic!("Expected ScopeDefinition statement"),
    }
}

#[test]
fn test_parse_scope_usage() {
    let lexer = Lexer::new("$ | @items := $[items] | @items[0]");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 2);
    assert!(matches!(query.statements[0], Statement::ScopeDefinition { .. }));
    assert!(matches!(query.statements[1], Statement::Access(_)));
}

// ============================================================================
// Access Statements
// ============================================================================

#[test]
fn test_parse_access_statement() {
    let lexer = Lexer::new("$ | $[items][0]");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    match &query.statements[0] {
        Statement::Access(expr) => {
            assert!(matches!(expr, Expr::Access { .. }));
        }
        _ => panic!("Expected Access statement"),
    }
}

// ============================================================================
// Output Statements
// ============================================================================

#[test]
fn test_parse_output_root() {
    let lexer = Lexer::new("$ | ?($[x] > 5) | !($)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    assert!(query.output.is_some());
    assert!(matches!(query.output.unwrap(), Expr::Root));
}

#[test]
fn test_parse_output_field() {
    let lexer = Lexer::new("$ | !($[items])");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 0);
    assert!(query.output.is_some());
    assert!(matches!(query.output.unwrap(), Expr::Access { .. }));
}

#[test]
fn test_parse_output_object() {
    let lexer = Lexer::new(r##"$ | !({"total": $[total], "count": $[count]})"##);
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert!(query.output.is_some());
    assert!(matches!(query.output.unwrap(), Expr::Object(_)));
}

#[test]
fn test_parse_no_output() {
    let lexer = Lexer::new("$ | ?($[x] > 5)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 1);
    assert!(query.output.is_none()); // Defaults to root
}

// ============================================================================
// Multiple Statements (Pipelines)
// ============================================================================

#[test]
fn test_parse_multiple_statements() {
    let lexer = Lexer::new("$ | ?($[status] == \"active\") | ~($[price] := $[price] * 1.1) | !($)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 2);
    assert!(matches!(query.statements[0], Statement::Filter(_)));
    assert!(matches!(query.statements[1], Statement::Transform { .. }));
    assert!(query.output.is_some());
}

#[test]
fn test_parse_complex_pipeline() {
    let lexer = Lexer::new(
        "$ | @items := $[items] | ?(@items[0][price] > 100) | ~($[total] := @items[0][price]) | !($[total])"
    );
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.statements.len(), 3);
}

// ============================================================================
// UDF Definitions
// ============================================================================

#[test]
fn test_parse_query_with_udf() {
    let lexer = Lexer::new("&expensive:1 := ?(@1[price] > 100)\n$ | ?($[items][0][price] > 50)");
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.udfs.len(), 1);
    assert_eq!(query.udfs[0].name, "expensive");
    assert_eq!(query.statements.len(), 1);
}

#[test]
fn test_parse_multiple_udfs() {
    let lexer = Lexer::new(
        "&expensive:1 := ?(@1[price] > 100)\n\
         &cheap:1 := ?(@1[price] < 50)\n\
         $ | ?($[x] > 5)"
    );
    let mut parser = Parser::new(lexer);
    let query = parser.parse_query();
    
    assert_eq!(query.udfs.len(), 2);
    assert_eq!(query.udfs[0].name, "expensive");
    assert_eq!(query.udfs[1].name, "cheap");
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_filter_missing_parens() {
    let lexer = Lexer::new("$ | ?$[x] > 5");
    let mut parser = Parser::new(lexer);
    parser.parse_query();
}

#[test]
#[should_panic(expected = "Expected")]
fn test_parse_transform_missing_assignment() {
    let lexer = Lexer::new("$ | ~($[price])");
    let mut parser = Parser::new(lexer);
    parser.parse_query();
}

#[test]
#[should_panic(expected = "Expected Dollar")]
fn test_parse_query_no_dollar_start() {
    let lexer = Lexer::new("?($[x] > 5)");
    let mut parser = Parser::new(lexer);
    parser.parse_query();
}

#[test]
#[should_panic(expected = "Expected identifier after `@`")]
fn test_parse_scope_no_name() {
    let lexer = Lexer::new("$ | @ := $[items]");
    let mut parser = Parser::new(lexer);
    parser.parse_query();
}

