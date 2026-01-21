#[cfg(test)]
mod tests {
    use clove_lang::*;
    use clove_lang::ast::BinOp;
    use clove_lang::evaluator::EvalError;
    use clove_lang::transform::{PathSegment, extract_path, TransformType, determine_transform_type, uses_lambda_param};

    // Helper functions to build AST for testing
    fn field(name: &str) -> Expr {
        Expr::Key(name.to_string())
    }

    fn string(s: &str) -> Expr {
        Expr::String(s.to_string())
    }

    fn number(n: i64) -> Expr {
        Expr::Integer(n)
    }

    fn float(n: f64) -> Expr {
        Expr::Float(n)
    }

    fn access(object: Expr, key: Expr) -> Expr {
        Expr::Access {
            object: Box::new(object),
            key: Box::new(key),
        }
    }

    // ========================================================================
    // Valid Path Extraction Tests
    // ========================================================================

    #[test]
    fn test_extract_single_field() {
        // $[name]
        let expr = access(Expr::Root, field("name"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 1);
        assert_eq!(path[0], PathSegment::Field("name".into()));
    }

    #[test]
    fn test_extract_nested_fields() {
        // $[user][name]
        let expr = access(access(Expr::Root, field("user")), field("name"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 2);
        assert_eq!(path[0], PathSegment::Field("user".into()));
        assert_eq!(path[1], PathSegment::Field("name".into()));
    }

    #[test]
    fn test_extract_array_index() {
        // $[items][0]
        let expr = access(access(Expr::Root, field("items")), number(0));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 2);
        assert_eq!(path[0], PathSegment::Field("items".into()));
        assert_eq!(path[1], PathSegment::Index(0));
    }

    #[test]
    fn test_extract_multiple_indices() {
        // $[matrix][5][10]
        let expr = access(
            access(access(Expr::Root, field("matrix")), number(5)),
            number(10),
        );
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 3);
        assert_eq!(path[0], PathSegment::Field("matrix".into()));
        assert_eq!(path[1], PathSegment::Index(5));
        assert_eq!(path[2], PathSegment::Index(10));
    }

    #[test]
    fn test_extract_complex_nested_path() {
        // $[users][0][profile][settings]
        let expr = access(
            access(
                access(access(Expr::Root, field("users")), number(0)),
                field("profile"),
            ),
            field("settings"),
        );
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 4);
        assert_eq!(path[0], PathSegment::Field("users".into()));
        assert_eq!(path[1], PathSegment::Index(0));
        assert_eq!(path[2], PathSegment::Field("profile".into()));
        assert_eq!(path[3], PathSegment::Field("settings".into()));
    }

    #[test]
    fn test_extract_quoted_field() {
        // $["@timestamp"]
        let expr = access(Expr::Root, string("@timestamp"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 1);
        assert_eq!(path[0], PathSegment::Field("@timestamp".into()));
    }

    #[test]
    fn test_extract_dotted_field_literal() {
        // $["user.email"] - single field with literal dot
        let expr = access(Expr::Root, string("user.email"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 1);
        assert_eq!(path[0], PathSegment::Field("user.email".into()));
    }

    #[test]
    fn test_extract_special_characters_in_field() {
        // $["field-with-hyphens"]
        let expr = access(Expr::Root, string("field-with-hyphens"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 1);
        assert_eq!(path[0], PathSegment::Field("field-with-hyphens".into()));
    }

    #[test]
    fn test_extract_mixed_access() {
        // $[items][0]["@timestamp"][data]
        let expr = access(
            access(
                access(access(Expr::Root, field("items")), number(0)),
                string("@timestamp"),
            ),
            field("data"),
        );
        let path = extract_path(&expr).unwrap();

        assert_eq!(path.len(), 4);
        assert_eq!(path[0], PathSegment::Field("items".into()));
        assert_eq!(path[1], PathSegment::Index(0));
        assert_eq!(path[2], PathSegment::Field("@timestamp".into()));
        assert_eq!(path[3], PathSegment::Field("data".into()));
    }

    #[test]
    fn test_extract_large_index() {
        // $[items][999]
        let expr = access(access(Expr::Root, field("items")), number(999));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path[1], PathSegment::Index(999));
    }

    // ========================================================================
    // Error Cases - Invalid Indices
    // ========================================================================

    #[test]
    fn test_reject_float_index() {
        // $[items][1.5]
        let expr = access(access(Expr::Root, field("items")), float(1.5));

        let result = &extract_path(&expr).unwrap()[1];
        // assert!(result);

        match result {
            PathSegment::Field(s) => assert!(s.parse::<f64>() == Ok(1.5_f64), "Did not register float access key as field key, registered as: {:?}", s),
            _ => panic!("Expected TypeError about float index"),
        };
    }

    #[test]
    fn test_reject_very_small_negative() {
        // $[items][-0.1]
        let expr = access(access(Expr::Root, field("items")), number(-1));

        let result = &extract_path(&expr).unwrap()[1];

        match result {
            PathSegment::Index(n) => {
                assert!(*n == -1, "Did not parse access key as correct negative value.")
            }
            _ => panic!("Did not parse negative index as index path segment")
        }

    }

    // ========================================================================
    // Error Cases - Invalid Target Types
    // ========================================================================

    #[test]
    fn test_reject_scope_reference() {
        // @items - scope reference not allowed
        let expr = Expr::ScopeRef("items".into());

        let result = extract_path(&expr);
        assert!(result.is_err());

        match result {
            Err(EvalError::TypeError(msg)) => {
                assert!(msg.contains("scope reference"), "Error message: {}", msg);
                assert!(msg.contains("@items"), "Error message: {}", msg);
            }
            _ => panic!("Expected TypeError about scope reference"),
        }
    }

    #[test]
    fn test_reject_scope_ref_in_path() {
        // @items[0] - scope ref with access
        let expr = access(Expr::ScopeRef("items".into()), number(0));

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_computed_key() {
        // $[items][$[index]] - dynamic key not allowed
        let expr = access(
            access(Expr::Root, field("items")),
            access(Expr::Root, field("index")),
        );

        let result = extract_path(&expr);
        assert!(result.is_err());

        match result {
            Err(EvalError::TypeError(msg)) => {
                assert!(msg.contains("computed"), "Error message: {}", msg);
            }
            _ => panic!("Expected TypeError about computed keys"),
        }
    }

    #[test]
    fn test_reject_expression_as_key() {
        // $[items][1 + 2] - expression in key position
        let expr = access(
            access(Expr::Root, field("items")),
            Expr::BinaryOp {
                op: BinOp::Add,
                left: Box::new(number(1)),
                right: Box::new(number(2)),
            },
        );

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_literal_number() {
        // 42 - not a path at all
        let expr = number(42);

        let result = extract_path(&expr);
        assert!(result.is_err());

        match result {
            Err(EvalError::TypeError(msg)) => {
                assert!(
                    msg.contains("Invalid transform target"),
                    "Error message: {}",
                    msg
                );
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_reject_string_literal() {
        // "hello" - not a path
        let expr = string("hello");

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_binary_operation() {
        // $[a] + $[b] - binary op not a valid target
        let expr = Expr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(access(Expr::Root, field("a"))),
            right: Box::new(access(Expr::Root, field("b"))),
        };

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_lambda_param() {
        // @ - lambda parameter not a valid target
        let expr = Expr::LambdaParam;

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_env_var() {
        // $HOME - env var not a valid target
        let expr = Expr::EnvVar("HOME".into());

        let result = extract_path(&expr);
        assert!(result.is_err());
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_extract_zero_index() {
        // $[items][0] - zero is valid
        let expr = access(access(Expr::Root, field("items")), number(0));

        let path = extract_path(&expr).unwrap();
        assert_eq!(path[1], PathSegment::Index(0));
    }

    #[test]
    fn test_extract_single_character_field() {
        // $[x]
        let expr = access(Expr::Root, field("x"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path[0], PathSegment::Field("x".into()));
    }

    #[test]
    fn test_extract_empty_string_field() {
        // $[""]
        let expr = access(Expr::Root, string(""));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path[0], PathSegment::Field("".into()));
    }

    #[test]
    fn test_extract_unicode_field() {
        // $["日本語"]
        let expr = access(Expr::Root, string("日本語"));
        let path = extract_path(&expr).unwrap();

        assert_eq!(path[0], PathSegment::Field("日本語".into()));
    }

    #[test]
    fn test_path_segment_equality() {
        assert_eq!(
            PathSegment::Field("test".into()),
            PathSegment::Field("test".into())
        );
        assert_ne!(
            PathSegment::Field("test".into()),
            PathSegment::Field("other".into())
        );
        assert_eq!(PathSegment::Index(5), PathSegment::Index(5));
        assert_ne!(PathSegment::Index(5), PathSegment::Index(6));
        assert_ne!(PathSegment::Field("5".into()), PathSegment::Index(5));
    }
    
    
    // ========================================================================
    // Transform Type Detection Tests
    // ========================================================================
    
    mod transform_type_tests {
        use super::*;
        
        // Helper functions
        fn field(name: &str) -> Expr {
            Expr::Key(name.to_string())
        }
        
        fn string(s: &str) -> Expr {
            Expr::String(s.to_string())
        }
        
        fn number(n: i64) -> Expr {
            Expr::Integer(n)
        }

        fn access(object: Expr, key: Expr) -> Expr {
            Expr::Access {
                object: Box::new(object),
                key: Box::new(key),
            }
        }
        
        fn binop(op: BinOp, left: Expr, right: Expr) -> Expr {
            Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        
        // ====================================================================
        // FilterArray Detection
        // ====================================================================
        
        #[test]
        fn test_detect_filter_simple() {
            // ?(@[price] > 100)
            let condition = binop(
                BinOp::GreaterThan,
                access(Expr::LambdaParam, field("price")),
                number(100)
            );
            let expr = Expr::Filter(Box::new(condition.clone()));
            
            let transform_type = determine_transform_type(&expr);
            
            match transform_type {
                TransformType::FilterArray(cond) => {
                    // Check it extracted the condition correctly
                    assert!(matches!(cond, Expr::BinaryOp { op: BinOp::GreaterThan, .. }));
                }
                _ => panic!("Expected FilterArray, got {:?}", transform_type),
            }
        }
        
        #[test]
        fn test_detect_filter_complex() {
            // ?(@[status] == "active" and @[verified] == true)
            let condition = binop(
                BinOp::And,
                binop(
                    BinOp::Equal,
                    access(Expr::LambdaParam, field("status")),
                    string("active")
                ),
                binop(
                    BinOp::Equal,
                    access(Expr::LambdaParam, field("verified")),
                    Expr::Boolean(true)
                )
            );
            let expr = Expr::Filter(Box::new(condition));
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::FilterArray(_)));
        }
        
        // ====================================================================
        // MapArray Detection
        // ====================================================================
        
        #[test]
        fn test_detect_map_simple_field_access() {
            // @[price]
            let expr = access(Expr::LambdaParam, field("price"));
            
            let transform_type = determine_transform_type(&expr);
            
            match transform_type {
                TransformType::MapArray(map_expr) => {
                    assert!(uses_lambda_param(&map_expr));
                }
                _ => panic!("Expected MapArray, got {:?}", transform_type),
            }
        }
        
        #[test]
        fn test_detect_map_nested_access() {
            // @[user][name]
            let expr = access(
                access(Expr::LambdaParam, field("user")),
                field("name")
            );
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::MapArray(_)));
        }
        
        #[test]
        fn test_detect_map_with_arithmetic() {
            // @[price] * 1.1
            let expr = binop(
                BinOp::Multiply,
                access(Expr::LambdaParam, field("price")),
                number(1)
            );
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::MapArray(_)));
        }
        
        #[test]
        fn test_detect_map_with_string_concat() {
            // @[first] + " " + @[last]
            let expr = binop(
                BinOp::Add,
                binop(
                    BinOp::Add,
                    access(Expr::LambdaParam, field("first")),
                    string(" ")
                ),
                access(Expr::LambdaParam, field("last"))
            );
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::MapArray(_)));
        }
        
        #[test]
        fn test_detect_map_bare_lambda() {
            // @ (return entire item)
            let expr = Expr::LambdaParam;
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::MapArray(_)));
        }
        
        #[test]
        fn test_detect_map_lambda_in_comparison() {
            // @[x] > 5 (not a filter, just a map to boolean)
            let expr = binop(
                BinOp::GreaterThan,
                access(Expr::LambdaParam, field("x")),
                number(5)
            );
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::MapArray(_)));
        }
        
        // ====================================================================
        // Replace Detection
        // ====================================================================
        
        #[test]
        fn test_detect_replace_literal_number() {
            // 42
            let expr = number(42);
            
            let transform_type = determine_transform_type(&expr);
            
            match transform_type {
                TransformType::Replace(repl_expr) => {
                    assert!(matches!(repl_expr, Expr::Integer(n) if n == 42));
                }
                _ => panic!("Expected Replace, got {:?}", transform_type),
            }
        }
        
        #[test]
        fn test_detect_replace_literal_string() {
            // "hello"
            let expr = string("hello");
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_root_access() {
            // $[other_field]
            let expr = access(Expr::Root, field("other_field"));
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_arithmetic_no_lambda() {
            // $[a] + $[b]
            let expr = binop(
                BinOp::Add,
                access(Expr::Root, field("a")),
                access(Expr::Root, field("b"))
            );
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_scope_ref() {
            // @items (scope reference, not lambda param)
            let expr = Expr::ScopeRef("items".into());
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_env_var() {
            // $HOME
            let expr = Expr::EnvVar("HOME".into());
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_object_literal() {
            // {"x": 5, "y": 10}
            let expr = Expr::Object(vec![
                ("x".into(), number(5)),
                ("y".into(), number(10)),
            ]);
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        #[test]
        fn test_detect_replace_array_literal() {
            // [1, 2, 3]
            let expr = Expr::Array(vec![number(1), number(2), number(3)]);
            
            let transform_type = determine_transform_type(&expr);
            assert!(matches!(transform_type, TransformType::Replace(_)));
        }
        
        // ====================================================================
        // uses_lambda_param Tests
        // ====================================================================
        
        #[test]
        fn test_uses_lambda_bare() {
            assert!(uses_lambda_param(&Expr::LambdaParam));
        }
        
        #[test]
        fn test_uses_lambda_in_access_object() {
            // @[field]
            let expr = access(Expr::LambdaParam, field("field"));
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_access_key() {
            // This is weird but theoretically possible: $[@]
            let expr = access(Expr::Root, Expr::LambdaParam);
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_binop_left() {
            // @ + 5
            let expr = binop(BinOp::Add, Expr::LambdaParam, number(5));
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_binop_right() {
            // 5 + @
            let expr = binop(BinOp::Add, number(5), Expr::LambdaParam);
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_deeply_nested() {
            // (($[x] + @[y]) * 3) - 7
            let expr = binop(
                BinOp::Subtract,
                binop(
                    BinOp::Multiply,
                    binop(
                        BinOp::Add,
                        access(Expr::Root, field("x")),
                        access(Expr::LambdaParam, field("y"))
                    ),
                    number(3)
                ),
                number(7)
            );
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_object() {
            // {"x": @[price], "y": 10}
            let expr = Expr::Object(vec![
                ("x".into(), access(Expr::LambdaParam, field("price"))),
                ("y".into(), number(10)),
            ]);
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_array() {
            // [1, @[x], 3]
            let expr = Expr::Array(vec![
                number(1),
                access(Expr::LambdaParam, field("x")),
                number(3),
            ]);
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_uses_lambda_in_filter() {
            // ?(@[x] > 5)
            let condition = binop(
                BinOp::GreaterThan,
                access(Expr::LambdaParam, field("x")),
                number(5)
            );
            let expr = Expr::Filter(Box::new(condition));
            assert!(uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_not_uses_lambda_literals() {
            assert!(!uses_lambda_param(&Expr::Null));
            assert!(!uses_lambda_param(&Expr::Boolean(true)));
            assert!(!uses_lambda_param(&number(42)));
            assert!(!uses_lambda_param(&string("hello")));
        }
        
        #[test]
        fn test_not_uses_lambda_root() {
            assert!(!uses_lambda_param(&Expr::Root));
        }
        
        #[test]
        fn test_not_uses_lambda_scope_ref() {
            assert!(!uses_lambda_param(&Expr::ScopeRef("items".into())));
        }
        
        #[test]
        fn test_not_uses_lambda_env_var() {
            assert!(!uses_lambda_param(&Expr::EnvVar("HOME".into())));
        }
        
        #[test]
        fn test_not_uses_lambda_pure_root_access() {
            // $[field][nested]
            let expr = access(
                access(Expr::Root, field("field")),
                field("nested")
            );
            assert!(!uses_lambda_param(&expr));
        }
        
        #[test]
        fn test_not_uses_lambda_pure_arithmetic() {
            // $[a] + $[b] * 5
            let expr = binop(
                BinOp::Add,
                access(Expr::Root, field("a")),
                binop(
                    BinOp::Multiply,
                    access(Expr::Root, field("b")),
                    number(5)
                )
            );
            assert!(!uses_lambda_param(&expr));
        }
        
        // ====================================================================
        // Edge Cases
        // ====================================================================
        
        #[test]
        fn test_transform_type_equality() {
            let expr1 = number(5);
            let expr2 = number(5);
            
            let type1 = determine_transform_type(&expr1);
            let type2 = determine_transform_type(&expr2);
            
            // Can compare transform types
            assert_eq!(type1, type2);
        }
        
        #[test]
        fn test_filter_vs_map_distinction() {
            // Map: @[x] > 5 (evaluates to boolean for each element)
            let map_expr = binop(
                BinOp::GreaterThan,
                access(Expr::LambdaParam, field("x")),
                number(5)
            );
            
            // Filter: ?(@[x] > 5) (filters elements)
            let filter_expr = Expr::Filter(Box::new(map_expr.clone()));
            
            let map_type = determine_transform_type(&map_expr);
            let filter_type = determine_transform_type(&filter_expr);
            
            assert!(matches!(map_type, TransformType::MapArray(_)));
            assert!(matches!(filter_type, TransformType::FilterArray(_)));
            assert_ne!(map_type, filter_type);
        }
    }
}
