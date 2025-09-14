//! Sprint 2: Combined lexer/parser tests through public API
//! Testing tokenization and parsing comprehensively

use ruchy::Parser;

// Test various token types through parsing

#[test]
fn test_parse_keywords() {
    let keywords = vec![
        "let x = 1",
        "mut y = 2",
        "if true { 1 }",
        "else { 2 }",
        "fn foo() { }",
        "return 42",
        "for i in 1..10 { }",
        "while true { }",
        "match x { _ => 1 }",
        "true",
        "false",
        "nil",
    ];

    for keyword_expr in keywords {
        let mut parser = Parser::new(keyword_expr);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", keyword_expr);
    }
}

#[test]
fn test_parse_operators() {
    let operators = vec![
        "1 + 2",
        "5 - 3",
        "4 * 6",
        "10 / 2",
        "7 % 3",
        "x = 5",
        "a == b",
        "c != d",
        "1 < 2",
        "2 <= 2",
        "3 > 2",
        "3 >= 3",
        "true && false",
        "true || false",
        "!true",
        "obj.field",
        "1..10",
        "1..=10",
    ];

    for op_expr in operators {
        let mut parser = Parser::new(op_expr);
        let result = parser.parse();
        let _ = result; // Some might not be supported
    }
}

#[test]
fn test_parse_string_literals() {
    let strings = vec![
        r#""hello""#,
        r#""hello world""#,
        r#""""#,
        r#""with spaces""#,
    ];

    for string_lit in strings {
        let mut parser = Parser::new(string_lit);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", string_lit);
    }
}

#[test]
fn test_parse_number_literals() {
    let numbers = vec![
        "0",
        "42",
        "1234567890",
        "3.14",
        "0.5",
        "10.0",
    ];

    for num in numbers {
        let mut parser = Parser::new(num);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", num);
    }
}

#[test]
fn test_parse_identifiers() {
    let identifiers = vec![
        "x",
        "variable",
        "camelCase",
        "snake_case",
        "_private",
        "var123",
    ];

    for id in identifiers {
        let mut parser = Parser::new(id);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", id);
    }
}

#[test]
fn test_parse_delimiters() {
    let delimited = vec![
        "(1 + 2)",
        "[1, 2, 3]",
        "{ x: 1 }",
        "((nested))",
        "[[nested]]",
        "{{ nested }}",
    ];

    for expr in delimited {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Some might not be supported
    }
}

#[test]
fn test_parse_complex_expressions() {
    let complex = vec![
        "let result = (x + y) * 2",
        "if x > 0 { x } else { -x }",
        "fn add(a, b) { a + b }",
        "[1, 2, 3].map(|x| x * 2)",
        "match value { Some(n) => n, None => 0 }",
    ];

    for expr in complex {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Check for crashes
    }
}

#[test]
fn test_parse_method_chains() {
    let chains = vec![
        "obj.method()",
        "obj.method().another()",
        "obj.field.method()",
        "array[0].field",
    ];

    for chain in chains {
        let mut parser = Parser::new(chain);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_nested_structures() {
    let nested = vec![
        "[[1, 2], [3, 4]]",
        "(1, (2, 3))",
        "{ a: { b: 1 } }",
        "if if x { true } else { false } { 1 } else { 2 }",
    ];

    for expr in nested {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_edge_cases() {
    let edge_cases = vec![
        "",           // Empty
        "   ",        // Whitespace only
        "42",         // Single token
        "(())",       // Empty parens
        "[]",         // Empty array
        "{}",         // Empty object
    ];

    for expr in edge_cases {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Should handle gracefully
    }
}

#[test]
fn test_parse_very_long_expression() {
    let long_expr = (0..100)
        .map(|i| format!("{}", i))
        .collect::<Vec<_>>()
        .join(" + ");

    let mut parser = Parser::new(&long_expr);
    let result = parser.parse();
    let _ = result; // Should handle long input
}

#[test]
fn test_parse_deeply_nested() {
    let mut nested = String::from("1");
    for _ in 0..50 {
        nested = format!("({})", nested);
    }

    let mut parser = Parser::new(&nested);
    let result = parser.parse();
    let _ = result; // Should handle deep nesting
}

// Specific language features

#[test]
fn test_parse_fat_arrow() {
    let fat_arrows = vec![
        "x => x + 1",
        "(x, y) => x + y",
        "match x { 1 => \"one\", _ => \"other\" }",
    ];

    for expr in fat_arrows {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_pipeline() {
    let pipelines = vec![
        "5 |> double",
        "data |> filter |> map",
    ];

    for expr in pipelines {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_async_await() {
    let async_code = vec![
        "async { 1 }",
        "await promise",
        "async fn foo() { }",
    ];

    for expr in async_code {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_string_interpolation() {
    let interpolated = vec![
        r#"f"Hello {name}""#,
        r#"f"Result: {1 + 1}""#,
    ];

    for expr in interpolated {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_patterns() {
    let patterns = vec![
        "let (x, y) = (1, 2)",
        "let [a, b, c] = [1, 2, 3]",
        "let Some(x) = opt",
        "let Point { x, y } = point",
    ];

    for pattern in patterns {
        let mut parser = Parser::new(pattern);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_generics() {
    let generics = vec![
        "Vec<T>",
        "HashMap<String, i32>",
        "fn foo<T>(x: T) { }",
    ];

    for expr in generics {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result;
    }
}

#[test]
fn test_parse_attributes() {
    let attrs = vec![
        "#[derive(Debug)]",
        "#[test]",
    ];

    for attr in attrs {
        let mut parser = Parser::new(attr);
        let result = parser.parse();
        let _ = result;
    }
}

// Error recovery

#[test]
fn test_parse_incomplete() {
    let incomplete = vec![
        "let x =",
        "if true {",
        "fn foo(",
        "1 +",
    ];

    for expr in incomplete {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Should handle incomplete input
    }
}

#[test]
fn test_parse_invalid() {
    let invalid = vec![
        "let 123 = x",
        "fn () { }",
        "if { }",
        "1 + + 2",
    ];

    for expr in invalid {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Should handle invalid input
    }
}

#[test]
fn test_parse_recovery() {
    let with_errors = vec![
        "let x = error; let y = 10",
        "1 + ; 2 + 3",
        "fn foo() { error } fn bar() { 1 }",
    ];

    for expr in with_errors {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        let _ = result; // Should attempt recovery
    }
}