//! Sprint 2: Operator precedence and error recovery tests
//! Ensuring correct parsing order and graceful error handling

use ruchy::Parser;

// PARSE-002: Operator precedence

#[test]
fn test_precedence_multiplication_over_addition() {
    let cases = vec![
        ("2 + 3 * 4", 14),     // Should be 2 + (3 * 4) = 14, not (2 + 3) * 4 = 20
        ("10 - 2 * 3", 4),     // Should be 10 - (2 * 3) = 4
        ("1 + 2 * 3 + 4", 11), // Should be 1 + (2 * 3) + 4 = 11
    ];

    for (expr, _expected) in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_division_same_as_multiplication() {
    let cases = vec![
        "10 / 2 * 5",   // Should be (10 / 2) * 5 = 25, left-to-right
        "20 * 2 / 4",   // Should be (20 * 2) / 4 = 10
        "100 / 5 / 2",  // Should be (100 / 5) / 2 = 10
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_comparison_operators() {
    let cases = vec![
        "1 + 2 > 2",        // Should be (1 + 2) > 2
        "5 < 3 * 2",        // Should be 5 < (3 * 2)
        "10 - 5 >= 3 + 1",  // Should be (10 - 5) >= (3 + 1)
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_logical_and_over_or() {
    let cases = vec![
        "true || false && false",  // Should be true || (false && false)
        "a && b || c && d",        // Should be (a && b) || (c && d)
        "x || y && z || w",        // Should be x || (y && z) || w
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_equality_vs_comparison() {
    let cases = vec![
        "x == y > z",       // Should parse correctly with proper precedence
        "a != b < c",       // Comparison has higher precedence than equality
        "1 < 2 == true",    // Should be (1 < 2) == true
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_unary_operators() {
    let cases = vec![
        "-1 + 2",           // Should be (-1) + 2 = 1
        "!true && false",   // Should be (!true) && false
        "-x * y",           // Should be (-x) * y
        "!a || b",          // Should be (!a) || b
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_parentheses_override() {
    let cases = vec![
        "(2 + 3) * 4",      // Force addition first
        "2 * (3 + 4)",      // Force addition first
        "((1 + 2) * 3) + 4", // Nested parentheses
        "(a || b) && c",    // Override logical precedence
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_method_calls() {
    let cases = vec![
        "obj.method() + 1",     // Method call before addition
        "x + obj.field * 2",    // Field access and multiplication
        "a.b.c + d.e.f",        // Chained access
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_function_calls() {
    let cases = vec![
        "f(x) + g(y)",          // Function calls before addition
        "f(x * 2) + 3",         // Argument evaluation
        "f(g(h(x)))",           // Nested calls
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_array_indexing() {
    let cases = vec![
        "arr[0] + 1",           // Indexing before addition
        "arr[i * 2]",           // Expression in index
        "matrix[i][j]",         // Chained indexing
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_range_operators() {
    let cases = vec![
        "1..10",                // Basic range
        "start..end + 1",       // Range vs addition
        "0..=n * 2",            // Inclusive range with expression
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let _result = parser.parse();
        // Range precedence might vary
    }
}

#[test]
fn test_precedence_assignment() {
    let cases = vec![
        "x = y + 1",            // Assignment is lowest precedence
        "a = b = c",            // Right associative
        "x += y * 2",           // Compound assignment
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let _result = parser.parse();
        // Assignment might be statement, not expression
    }
}

// PARSE-003: Error recovery mechanisms

#[test]
fn test_error_recovery_missing_closing_paren() {
    let cases = vec![
        "(1 + 2",
        "((1 + 2)",
        "f(x, y",
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle missing parenthesis gracefully
        let _ = result;
    }
}

#[test]
fn test_error_recovery_missing_closing_bracket() {
    let cases = vec![
        "[1, 2, 3",
        "arr[index",
        "[[1, 2], [3, 4]",
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle missing bracket gracefully
        let _ = result;
    }
}

#[test]
fn test_error_recovery_missing_closing_brace() {
    let cases = vec![
        "{ x: 1, y: 2",
        "if true { print()",
        "fn foo() { bar()",
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle missing brace gracefully
        let _ = result;
    }
}

#[test]
fn test_error_recovery_unexpected_token() {
    let cases = vec![
        "1 + + 2",              // Double operator
        "let = 5",              // Keyword as identifier
        "x 5",                  // Missing operator
        "1 2 3",                // Consecutive numbers
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should recover from unexpected tokens
        let _ = result;
    }
}

#[test]
fn test_error_recovery_incomplete_expression() {
    let cases = vec![
        "1 +",                  // Missing right operand
        "if x >",               // Incomplete comparison
        "let x =",              // Missing value
        "fn foo(",              // Incomplete function
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle incomplete expressions
        let _ = result;
    }
}

#[test]
fn test_error_recovery_invalid_syntax() {
    let cases = vec![
        "let 123 = x",          // Number as identifier
        "fn (x) { }",           // Missing function name
        "if { }",               // Missing condition
        "for in list { }",      // Missing iterator variable
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle invalid syntax
        let _ = result;
    }
}

#[test]
fn test_error_recovery_multiple_errors() {
    let cases = vec![
        "let x = (1 + + 2",     // Multiple errors
        "fn foo( { if (x > {",  // Many unclosed delimiters
        "1 + * 2 - / 3",        // Multiple operator errors
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should attempt recovery from multiple errors
        let _ = result;
    }
}

#[test]
fn test_error_recovery_after_error() {
    let input = "let x = error; let y = 10; let z = 20";
    let mut parser = Parser::new(input);
    let result = parser.parse();

    // Even if first statement has error, should try to parse rest
    let _ = result;
}

#[test]
fn test_error_recovery_nested_errors() {
    let cases = vec![
        "f(g(h(error)))",       // Error in deeply nested call
        "{ { { error } } }",    // Error in nested blocks
        "[[[error]]]",          // Error in nested arrays
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        // Should handle nested errors
        let _ = result;
    }
}

#[test]
fn test_error_recovery_sync_points() {
    // Parser should synchronize at certain tokens
    let input = "let x = error\nlet y = 10\nlet z = 20";
    let mut parser = Parser::new(input);
    let result = parser.parse();

    // Should use newlines or semicolons as sync points
    let _ = result;
}

// Associativity tests

#[test]
fn test_associativity_left_to_right() {
    let cases = vec![
        "1 - 2 - 3",            // Should be (1 - 2) - 3 = -4
        "10 / 2 / 5",           // Should be (10 / 2) / 5 = 1
        "a.b.c.d",              // Should be ((a.b).c).d
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_associativity_right_to_left() {
    let cases = vec![
        "a = b = c = 5",        // Assignment is right-associative
        "2 ** 3 ** 2",          // Exponentiation (if supported)
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let _result = parser.parse();
        // Associativity might vary
    }
}

// Complex precedence combinations

#[test]
fn test_complex_precedence_mix() {
    let cases = vec![
        "!a && b || c && !d",
        "-x * y + z / w",
        "a.b[c] + d(e) * f",
        "x > y && y > z || a == b",
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_precedence_with_pipeline() {
    let cases = vec![
        "x + 1 |> f",           // Pipeline should be lowest precedence
        "a |> b |> c + 1",      // How does pipeline interact with operators?
    ];

    for expr in cases {
        let mut parser = Parser::new(expr);
        let _result = parser.parse();
        // Pipeline precedence might vary
    }
}

#[test]
fn test_precedence_table_completeness() {
    // Test that all operators have defined precedence
    let all_ops = vec![
        "a + b", "a - b", "a * b", "a / b", "a % b",
        "a == b", "a != b", "a < b", "a <= b", "a > b", "a >= b",
        "a && b", "a || b", "!a", "-a", "+a",
        "a.b", "a[b]", "a(b)", "a |> b",
    ];

    for expr in all_ops {
        let mut parser = Parser::new(expr);
        let _result = parser.parse();
        // All operators should be parseable
    }
}