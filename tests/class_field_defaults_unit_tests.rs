//! EXTREME TDD: Unit tests for field default implementation
//! These tests verify specific behaviors and edge cases

use ruchy::{Parser, Transpiler};

#[test]
fn test_single_field_with_integer_default() {
    let code = r"
        class Point {
            x: i32 = 42
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();
    let result_str = result.to_string();

    // Should generate struct with field
    assert!(result_str.contains("struct Point"));
    assert!(result_str.contains("x : i32"));

    // Should generate impl block and Default trait implementation
    assert!(result_str.contains("impl Point"));
    assert!(result_str.contains("impl Default for Point"));
    assert!(result_str.contains("fn default"));
    assert!(result_str.contains("42")); // Default value should appear
}

#[test]
fn test_multiple_fields_with_mixed_defaults() {
    let code = r#"
        class Config {
            port: i32 = 8080,
            host: String = "localhost",
            debug: bool = false,
            timeout: i32
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct
    assert!(result_str.contains("struct Config"));
    assert!(result_str.contains("port : i32"));
    assert!(result_str.contains("host : String"));
    assert!(result_str.contains("debug : bool"));
    assert!(result_str.contains("timeout : i32"));

    // Should generate Default trait implementation with defaults
    assert!(result_str.contains("impl Config"));
    assert!(result_str.contains("impl Default for Config"));
    assert!(result_str.contains("fn default"));
    assert!(result_str.contains("8080"));
    assert!(result_str.contains("localhost"));
    assert!(result_str.contains("false"));
}

#[test]
fn test_field_default_with_expression() {
    let code = r"
        class Calculator {
            result: i32 = 10 + 5 * 2
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("struct Calculator"));
    assert!(result_str.contains("10 + 5 * 2"));
}

#[test]
fn test_field_default_with_method_call() {
    let code = r"
        class Buffer {
            data: String = String::new()
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("struct Buffer"));
    assert!(result_str.contains("String :: new ()"));
}

#[test]
fn test_constructor_instantiation_with_defaults() {
    let code = r#"
        class Person {
            name: String,
            age: i32 = 0
        }

        fun main() {
            Person { name: "Alice" }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should allow partial construction with defaults
    assert!(result_str.contains("Person { name : \"Alice\""));
}

#[test]
fn test_constructor_instantiation_override_defaults() {
    let code = r#"
        class Person {
            name: String,
            age: i32 = 0
        }

        fun main() {
            Person { name: "Bob", age: 25 }
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should allow overriding defaults
    assert!(result_str.contains("name : \"Bob\""));
    assert!(result_str.contains("age : 25"));
}

#[test]
fn test_empty_constructor_with_all_defaults() {
    let code = r#"
        class Settings {
            theme: String = "dark",
            volume: i32 = 50,
            muted: bool = false
        }

        fun main() {
            Settings {}
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should allow empty constructor when all fields have defaults
    assert!(result_str.contains("Settings { }"));
}

#[test]
fn test_field_defaults_parsing_failure_cases() {
    // Test cases that should fail during parsing

    // Missing type annotation with default
    let code1 = r"
        class Test {
            field = 42
        }
    ";

    let mut parser1 = Parser::new(code1);
    let result1 = parser1.parse();
    assert!(result1.is_err(), "Should fail without type annotation");

    // Invalid default value syntax
    let code2 = r"
        class Test {
            field: i32 =
        }
    ";

    let mut parser2 = Parser::new(code2);
    let result2 = parser2.parse();
    assert!(
        result2.is_err(),
        "Should fail with incomplete default value"
    );
}

#[test]
fn test_field_defaults_ast_structure() {
    let code = r"
        class Test {
            field1: i32 = 42,
            field2: String
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure contains default information
    if let ruchy::frontend::ast::ExprKind::Class { fields, .. } = &ast.kind {
        assert_eq!(fields.len(), 2, "Should have 2 fields");

        // First field should have default
        assert!(
            fields[0].default_value.is_some(),
            "First field should have default value"
        );

        // Second field should not have default
        assert!(
            fields[1].default_value.is_none(),
            "Second field should not have default value"
        );
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_field_defaults_complex_expressions() {
    let code = r"
        class Math {
            pi: f64 = 3.14159,
            e: f64 = 2.71828,
            golden_ratio: f64 = (1.0 + 5.0.sqrt()) / 2.0
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    assert!(result_str.contains("3.14159"));
    assert!(result_str.contains("2.71828"));
    assert!(result_str.contains("1f64 + 5f64")); // Transpiler uses f64 suffix
}
