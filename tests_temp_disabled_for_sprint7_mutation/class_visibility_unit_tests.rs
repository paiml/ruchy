//! EXTREME TDD: Unit tests for visibility modifier implementation
//! These tests verify specific visibility behaviors

use ruchy::{Parser, Transpiler};

#[test]
fn test_public_field_visibility() {
    let code = r"
        class Point {
            pub x: i32,
            y: i32
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with public field
    assert!(result_str.contains("struct Point"));
    assert!(result_str.contains("pub x : i32"));
    assert!(result_str.contains("y : i32"));
    // Private field should not have pub modifier
    assert!(!result_str.contains("pub y : i32"));
}

#[test]
fn test_mutable_field_modifier() {
    let code = r"
        class Counter {
            mut count: i32,
            name: String
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Struct should contain both fields (mut is about usage, not struct definition)
    assert!(result_str.contains("struct Counter"));
    assert!(result_str.contains("count : i32"));
    assert!(result_str.contains("name : String"));
}

#[test]
fn test_public_mutable_field() {
    let code = r"
        class Config {
            pub mut debug: bool,
            port: i32
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with public field
    assert!(result_str.contains("struct Config"));
    assert!(result_str.contains("pub debug : bool"));
    assert!(result_str.contains("port : i32"));
    assert!(!result_str.contains("pub port : i32"));
}

#[test]
fn test_public_method_visibility() {
    let code = r"
        class Calculator {
            pub fn add(&self, a: i32, b: i32) -> i32 {
                a + b
            }

            fn private_helper(&self) -> i32 {
                42
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate impl block with correct method visibility
    assert!(result_str.contains("impl Calculator"));
    assert!(result_str.contains("pub fn add"));
    assert!(result_str.contains("fn private_helper"));
    // Private method should not have pub
    assert!(!result_str.contains("pub fn private_helper"));
}

#[test]
fn test_public_constructor_visibility() {
    let code = r"
        class Person {
            name: String,

            pub new(name: String) {
                Person { name }
            }
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate impl block with public constructor
    assert!(result_str.contains("impl Person"));
    assert!(result_str.contains("pub fn new"));
}

#[test]
fn test_mixed_visibility_fields() {
    let code = r"
        class Server {
            pub port: i32,
            mut connections: i32,
            pub mut debug: bool,
            host: String
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with correct visibility
    assert!(result_str.contains("struct Server"));
    assert!(result_str.contains("pub port : i32"));
    assert!(result_str.contains("connections : i32"));
    assert!(result_str.contains("pub debug : bool"));
    assert!(result_str.contains("host : String"));

    // Check that private fields don't have pub
    assert!(!result_str.contains("pub connections : i32"));
    assert!(!result_str.contains("pub host : String"));
}

#[test]
fn test_field_with_visibility_and_defaults() {
    let code = r#"
        class Settings {
            pub theme: String = "dark",
            mut volume: i32 = 50,
            pub mut auto_save: bool = true,
            timeout: i32 = 30
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with visibility
    assert!(result_str.contains("pub theme : String"));
    assert!(result_str.contains("volume : i32"));
    assert!(result_str.contains("pub auto_save : bool"));
    assert!(result_str.contains("timeout : i32"));

    // Should generate Default trait with defaults
    assert!(result_str.contains("impl Default for Settings"));
    assert!(result_str.contains("dark"));
    assert!(result_str.contains("50"));
    assert!(result_str.contains("true"));
    assert!(result_str.contains("30"));
}

#[test]
fn test_visibility_ast_structure() {
    let code = r"
        class Test {
            pub field1: i32,
            mut field2: String,
            pub mut field3: bool,
            field4: f64
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure contains visibility information
    if let ruchy::frontend::ast::ExprKind::Class { fields, .. } = &ast.kind {
        assert_eq!(fields.len(), 4, "Should have 4 fields");

        // field1: pub, not mut
        assert!(fields[0].visibility.is_public(), "field1 should be public");
        assert!(!fields[0].is_mut, "field1 should not be mutable");

        // field2: not pub, mut
        assert!(
            !fields[1].visibility.is_public(),
            "field2 should not be public"
        );
        assert!(fields[1].is_mut, "field2 should be mutable");

        // field3: pub and mut
        assert!(fields[2].visibility.is_public(), "field3 should be public");
        assert!(fields[2].is_mut, "field3 should be mutable");

        // field4: neither pub nor mut
        assert!(
            !fields[3].visibility.is_public(),
            "field4 should not be public"
        );
        assert!(!fields[3].is_mut, "field4 should not be mutable");
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_visibility_parsing_failure_cases() {
    // Test invalid visibility combinations

    let code1 = r"
        class Test {
            mut pub field: i32  // Wrong order
        }
    ";

    let mut parser1 = Parser::new(code1);
    let result1 = parser1.parse();
    assert!(result1.is_err(), "Should fail with wrong modifier order");

    let code2 = r"
        class Test {
            pub pub field: i32  // Duplicate pub
        }
    ";

    let mut parser2 = Parser::new(code2);
    let result2 = parser2.parse();
    assert!(result2.is_err(), "Should fail with duplicate pub");
}
