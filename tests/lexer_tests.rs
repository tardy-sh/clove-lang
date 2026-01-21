// tests/lexer_tests.rs

use clove_lang::ast::Token;
use clove_lang::lexer::Lexer;

// ============================================================================
// Single Character Tokens
// ============================================================================

#[test]
fn test_single_char_tokens() {
    let test_cases = vec![
        ("$", Token::Dollar),
        ("@", Token::At),
        ("&", Token::Ampersand),
        ("?", Token::Question),
        ("~", Token::Tilde),
        ("!", Token::Exclamation),
        ("|", Token::Pipe),
        ("+", Token::Plus),
        ("-", Token::Minus),
        ("*", Token::Star),
        ("/", Token::Slash),
        ("%", Token::Percent),
        ("(", Token::LParen),
        (")", Token::RParen),
        ("[", Token::LBracket),
        ("]", Token::RBracket),
        ("{", Token::LBrace),
        ("}", Token::RBrace),
        (".", Token::Dot),
        (",", Token::Comma),
        (":", Token::Colon),
        ("<", Token::Lt),
        (">", Token::Gt),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert_eq!(token, expected, "Failed for input: {}", input);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

// ============================================================================
// Two Character Tokens
// ============================================================================

#[test]
fn test_two_char_tokens() {
    let test_cases = vec![
        ("==", Token::EqEq),
        ("!=", Token::NotEq),
        ("<=", Token::LtEq),
        (">=", Token::GtEq),
        (":=", Token::ColonEqual),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert_eq!(token, expected, "Failed for input: {}", input);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_two_char_vs_single_char() {
    // Valid: < followed by ==
    let mut lexer = Lexer::new("< ==");
    assert_eq!(lexer.next_token(), Token::Lt);
    assert_eq!(lexer.next_token(), Token::EqEq);
    assert_eq!(lexer.next_token(), Token::Eof);

    // Valid: <= as single token
    let mut lexer = Lexer::new("<=");
    assert_eq!(lexer.next_token(), Token::LtEq);
    assert_eq!(lexer.next_token(), Token::Eof);

    // Valid: < without space, then <=
    let mut lexer = Lexer::new("< <=");
    assert_eq!(lexer.next_token(), Token::Lt);
    assert_eq!(lexer.next_token(), Token::LtEq);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
#[should_panic(expected = "Unexpected '='")]
fn test_bare_equals_is_invalid() {
    let mut lexer = Lexer::new("< =");
    lexer.next_token(); // Gets 
    lexer.next_token(); // Should panic on bare =
}

// ============================================================================
// Keywords
// ============================================================================

#[test]
fn test_keywords() {
    let test_cases = vec![
        ("and", Token::And),
        ("or", Token::Or),
        ("true", Token::Boolean(true)),
        ("false", Token::Boolean(false)),
        ("null", Token::Null),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        let token = lexer.next_token();
        assert_eq!(token, expected, "Failed for input: {}", input);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_keywords_vs_identifiers() {
    // Make sure keywords only match when they're standalone words
    let test_cases = vec![
        ("android", "android"),
        ("and_item", "and_item"),
        ("_and", "_and"),
        ("or_gate", "or_gate"),
        ("order", "order"),
        ("truth", "truth"),
        ("false_positive", "false_positive"),
        ("nullable", "nullable"),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::Identifier(ident) => {
                assert_eq!(ident, expected, "Failed for input: {}", input);
            }
            other => panic!("Expected Identifier, got {:?} for input: {}", other, input),
        }
    }
}

// ============================================================================
// Identifiers
// ============================================================================

#[test]
fn test_identifiers() {
    let test_cases = vec![
        "x",
        "foo",
        "bar123",
        "snake_case",
        "camelCase",
        "PascalCase",
        "_private",
        "__dunder__",
        "a1b2c3",
        "item_count",
    ];

    for input in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::Identifier(ident) => {
                assert_eq!(ident, input, "Failed for input: {}", input);
            }
            other => panic!("Expected Identifier, got {:?} for input: {}", other, input),
        }
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

// ============================================================================
// Numbers
// ============================================================================

#[test]
fn test_integers() {
    let test_cases = vec![("0", 0), ("1", 1), ("42", 42), ("123456", 123456)];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::Integer(n) => {
                assert_eq!(n, expected, "Failed for input: {}", input);
            }
            other => panic!("Expected Number, got {:?} for input: {}", other, input),
        }
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_ints() {
    let test_cases = vec![
        ("0", 0),
        ("15", 15),
        ("315", 315),
        ("123456", 123456),
        ("01", 01),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::Integer(n) => {
                assert_eq!(
                    n, expected,
                    "Failed for input: {}, got {} expected {}",
                    input, n, expected
                );
            }
            other => panic!("Expected Number, got {:?} for input: {}", other, input),
        }
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_floats() {
    let test_cases = vec![
        ("0.0", 0.0),
        ("1.5", 1.5),
        ("3.15", 3.15),
        ("123.456", 123.456),
        ("0.1", 0.1),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::Float(n) => {
                assert!(
                    (n - expected).abs() < 0.0001,
                    "Failed for input: {}, got {} expected {}",
                    input,
                    n,
                    expected
                );
            }
            other => panic!("Expected Number, got {:?} for input: {}", other, input),
        }
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_negative_numbers() {
    let test_cases = vec![("-1", 1), ("-42", 42), ("-315", 315)];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        let mut result: Vec<Token> = vec![];
        loop {
            let token = lexer.next_token();
            if token == Token::Eof {
                break;
            }
            result.push(token);
        }
        assert_eq!(vec![Token::Minus, Token::Integer(expected)], result);
    }
}

#[test]
fn test_minus_vs_negative() {
    // "5-3" should be Number(5), Minus, Number(3)
    let mut lexer = Lexer::new("5-3");
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 5));
    assert_eq!(lexer.next_token(), Token::Minus);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 3));

    // "5 - 3" with spaces
    let mut lexer = Lexer::new("5 - 3.0");
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 5));
    assert_eq!(lexer.next_token(), Token::Minus);
    assert!(matches!(lexer.next_token(), Token::Float(n) if n == 3.0));
}

// ============================================================================
// Strings
// ============================================================================

#[test]
fn test_simple_strings() {
    let test_cases = vec![
        (r##""hello""##, "hello"),
        (r##""world""##, "world"),
        (r#""""#, ""),
        (r#""with spaces""#, "with spaces"),
        (r#""with-dashes""#, "with-dashes"),
        (r#""123""#, "123"),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::String(s) => {
                assert_eq!(s, expected, "Failed for input: {}", input);
            }
            other => panic!("Expected String, got {:?} for input: {}", other, input),
        }
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

#[test]
fn test_string_escapes() {
    let test_cases = vec![
        (r#""hello\nworld""#, "hello\nworld"),
        (r#""tab\there""#, "tab\there"),
        (r#""quote\"inside""#, "quote\"inside"),
        (r#""backslash\\here""#, "backslash\\here"),
        (r#""carriage\rreturn""#, "carriage\rreturn"),
        (r#""all\n\t\r\"\\together""#, "all\n\t\r\"\\together"),
    ];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::String(s) => {
                assert_eq!(s, expected, "Failed for input: {}", input);
            }
            other => panic!("Expected String, got {:?} for input: {}", other, input),
        }
    }
}

#[test]
fn test_single_quote_strings() {
    let test_cases = vec![("'hello'", "hello"), ("'world'", "world"), ("''", "")];

    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        match lexer.next_token() {
            Token::String(s) => {
                assert_eq!(s, expected, "Failed for input: {}", input);
            }
            other => panic!("Expected String, got {:?} for input: {}", other, input),
        }
    }
}

// ============================================================================
// Whitespace Handling
// ============================================================================

#[test]
fn test_whitespace_ignored() {
    let inputs = vec![
        "$[field]",
        "$ [ field ]",
        "  $  [  field  ]  ",
        "\t$\t[\tfield\t]\t",
        "\n$\n[\nfield\n]\n",
    ];

    for input in inputs {
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Dollar);
        assert_eq!(lexer.next_token(), Token::LBracket);
        match lexer.next_token() {
            Token::Identifier(s) if s == "field" => {}
            other => panic!("Expected Identifier(field), got {:?}", other),
        }
        assert_eq!(lexer.next_token(), Token::RBracket);
        assert_eq!(lexer.next_token(), Token::Eof);
    }
}

// ============================================================================
// Complex Token Sequences
// ============================================================================

#[test]
fn test_simple_access() {
    let mut lexer = Lexer::new("$[items][0]");
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "items"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 0));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_comparison_expression() {
    let mut lexer = Lexer::new("$[price] > 100");
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 100));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_logical_expression() {
    let mut lexer = Lexer::new("$[age] > 18 and $[verified] == true");

    // $[age] > 18
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "age"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 18));

    // and
    assert_eq!(lexer.next_token(), Token::And);

    // $[verified] == true
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "verified"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::EqEq);
    assert_eq!(lexer.next_token(), Token::Boolean(true));

    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_filter_syntax() {
    let mut lexer = Lexer::new(r#"$ | ?($[status] == "active")"#);

    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::Pipe);
    assert_eq!(lexer.next_token(), Token::Question);
    assert_eq!(lexer.next_token(), Token::LParen);
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "status"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::EqEq);
    assert!(matches!(lexer.next_token(), Token::String(s) if s == "active"));
    assert_eq!(lexer.next_token(), Token::RParen);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_transform_syntax() {
    let mut lexer = Lexer::new("~($[price] := $[price] * 1.1)");

    assert_eq!(lexer.next_token(), Token::Tilde);
    assert_eq!(lexer.next_token(), Token::LParen);
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::ColonEqual);
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Star);
    assert!(matches!(lexer.next_token(), Token::Float(n) if (n - 1.1).abs() < 0.0001));
    assert_eq!(lexer.next_token(), Token::RParen);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_scope_reference() {
    let mut lexer = Lexer::new("@items := $[items]");

    assert_eq!(lexer.next_token(), Token::At);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "items"));
    assert_eq!(lexer.next_token(), Token::ColonEqual);
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "items"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_method_call() {
    let mut lexer = Lexer::new("$[items].any(@[price] > 100)");

    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "items"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Dot);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "any"));
    assert_eq!(lexer.next_token(), Token::LParen);
    assert_eq!(lexer.next_token(), Token::At);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 100));
    assert_eq!(lexer.next_token(), Token::RParen);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_udf_definition() {
    let mut lexer = Lexer::new("&expensive,1 := ?(@1[price] > 100)");

    assert_eq!(lexer.next_token(), Token::Ampersand);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "expensive"));
    assert_eq!(lexer.next_token(), Token::Comma);
    assert!(matches!(lexer.next_token(), Token::Integer(n) if n == 1));
    // ... rest of tokens
}

#[test]
fn test_arithmetic() {
    let mut lexer = Lexer::new("$[a] + $[b] * $[c] - $[d] / $[e] % $[f]");

    let expected = vec![
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("a".to_string()),
        Token::RBracket,
        Token::Plus,
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("b".to_string()),
        Token::RBracket,
        Token::Star,
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("c".to_string()),
        Token::RBracket,
        Token::Minus,
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("d".to_string()),
        Token::RBracket,
        Token::Slash,
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("e".to_string()),
        Token::RBracket,
        Token::Percent,
        Token::Dollar,
        Token::LBracket,
        Token::Identifier("f".to_string()),
        Token::RBracket,
    ];

    for expected_token in expected {
        let token = lexer.next_token();
        assert_eq!(token, expected_token);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_input() {
    let mut lexer = Lexer::new("");
    assert_eq!(lexer.next_token(), Token::Eof);
    assert_eq!(lexer.next_token(), Token::Eof); // Should stay at EOF
}

#[test]
fn test_only_whitespace() {
    let mut lexer = Lexer::new("   \t\n\r   ");
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_consecutive_operators() {
    let mut lexer = Lexer::new("==!=");
    assert_eq!(lexer.next_token(), Token::EqEq);
    assert_eq!(lexer.next_token(), Token::NotEq);
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_no_space_between_tokens() {
    let mut lexer = Lexer::new("$[x]>5and$[y]<10");
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(_)));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert!(matches!(lexer.next_token(), Token::Integer(_)));
    assert_eq!(lexer.next_token(), Token::And);
    // etc...
}

// ============================================================================
// Error Cases (should panic)
// ============================================================================

#[test]
#[should_panic(expected = "Unterminated string")]
fn test_unterminated_string() {
    let mut lexer = Lexer::new(r##"'hello"##);
    lexer.next_token();
}

#[test]
#[should_panic(expected = "Unterminated string: unexpected EOF after backslash")]
fn test_unterminated_string_after_backslash() {
    let mut lexer = Lexer::new(r##"'hello\"##);
    lexer.next_token();
}

#[test]
#[should_panic(expected = "Invalid escape sequence")]
fn test_invalid_escape_sequence() {
    let mut lexer = Lexer::new(r#""hello\x""#);
    lexer.next_token();
}

#[test]
#[should_panic(expected = "Unexpected '='")]
fn test_bare_equals() {
    let mut lexer = Lexer::new("=");
    lexer.next_token();
}

#[test]
#[should_panic(expected = "Unexpected character")]
fn test_invalid_character() {
    let mut lexer = Lexer::new("#");
    lexer.next_token();
}

// ============================================================================
// Environment Variables
// ============================================================================

#[test]
fn test_env_var_vs_root() {
    // Environment variable
    let mut lexer = Lexer::new("$HOME");
    assert!(matches!(lexer.next_token(), Token::EnvVar(s) if s == "HOME"));

    // Root document
    let mut lexer = Lexer::new("$");
    assert_eq!(lexer.next_token(), Token::Dollar);

    // Root with access (NOT env var)
    let mut lexer = Lexer::new("$[HOME]");
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "HOME"));
}

#[test]
fn test_env_var_in_expression() {
    let mut lexer = Lexer::new("$[price] > $THRESHOLD");

    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert!(matches!(lexer.next_token(), Token::Identifier(s) if s == "price"));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert!(matches!(lexer.next_token(), Token::EnvVar(s) if s == "THRESHOLD"));
}

#[test]
fn test_lowercase_env_var() {
    let mut lexer = Lexer::new("$api_key");
    assert!(matches!(lexer.next_token(), Token::EnvVar(s) if s == "api_key"));
}
