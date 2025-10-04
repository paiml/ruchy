//! EXTREME TDD: Unit tests for derive attributes implementation
//! These tests verify specific derive attribute behaviors

use ruchy::{Parser, Transpiler};

#[test]
fn test_single_derive_debug() {
    let code = r"
        #[derive(Debug)]
        class Point {
            x: i32,
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

    // Should generate struct with Debug derive
    assert!(result_str.contains("struct Point"));
    // The quote! macro adds spaces, so check for the pattern with flexible spacing
    assert!(
        result_str.contains("# [derive (Debug)]") || result_str.contains("#[derive(Debug)]"),
        "Output should contain derive(Debug), got: {result_str}"
    );
    assert!(result_str.contains("x : i32"));
    assert!(result_str.contains("y : i32"));
}

#[test]
fn test_multiple_derives() {
    let code = r"
        #[derive(Debug, Clone, PartialEq)]
        class Config {
            name: String,
            value: i32
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should generate struct with multiple derives
    assert!(result_str.contains("struct Config"));
    assert!(
        result_str.contains("# [derive (Debug , Clone , PartialEq)]")
            || result_str.contains("#[derive(Debug, Clone, PartialEq)]")
    );
}

#[test]
fn test_derive_with_field_defaults() {
    let code = r#"
        #[derive(Debug)]
        class Settings {
            theme: String = "dark",
            volume: i32 = 50
        }
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should have both derive and Default trait
    assert!(result_str.contains("# [derive (Debug)]") || result_str.contains("#[derive(Debug)]"));
    assert!(result_str.contains("struct Settings"));
    assert!(result_str.contains("impl Default for Settings"));
}

#[test]
fn test_derive_with_visibility_modifiers() {
    let code = r"
        #[derive(Debug)]
        class Person {
            pub name: String,
            mut age: i32,
            pub mut active: bool
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should have derive with proper field visibility
    assert!(result_str.contains("# [derive (Debug)]") || result_str.contains("#[derive(Debug)]"));
    assert!(result_str.contains("pub name : String"));
    assert!(result_str.contains("age : i32"));
    assert!(result_str.contains("pub active : bool"));
}

#[test]
fn test_derive_with_methods() {
    let code = r"
        #[derive(Debug)]
        class Calculator {
            value: i32,

            pub fn add(&mut self, x: i32) -> i32 {
                self.value + x
            }

            fn reset(&mut self) {
                self.value = 0
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

    // Should have derive on struct and methods in impl
    assert!(result_str.contains("# [derive (Debug)]") || result_str.contains("#[derive(Debug)]"));
    assert!(result_str.contains("struct Calculator"));
    assert!(result_str.contains("impl Calculator"));
    assert!(result_str.contains("pub fn add"));
    assert!(result_str.contains("fn reset"));
}

#[test]
fn test_derive_common_traits() {
    let test_cases = vec![
        ("Debug", "#[derive(Debug)]"),
        ("Clone", "#[derive(Clone)]"),
        ("PartialEq", "#[derive(PartialEq)]"),
        ("Eq", "#[derive(Eq)]"),
        ("PartialOrd", "#[derive(PartialOrd)]"),
        ("Ord", "#[derive(Ord)]"),
        ("Hash", "#[derive(Hash)]"),
        ("Default", "#[derive(Default)]"),
        ("Copy", "#[derive(Copy)]"),
    ];

    for (trait_name, expected_derive) in test_cases {
        let code = format!(
            r"
            #[derive({trait_name})]
            class Test {{
                value: i32
            }}
            "
        );

        let mut parser = Parser::new(&code);
        let ast = parser
            .parse()
            .unwrap_or_else(|_| panic!("Should parse {trait_name} derive"));

        let transpiler = Transpiler::new();
        let result = transpiler
            .transpile(&ast)
            .unwrap_or_else(|_| panic!("Should transpile {trait_name} derive"));
        let result_str = result.to_string();

        // Check for various formatting versions from quote! macro
        let variations = vec![
            expected_derive.to_string(), // #[derive(Debug)]
            expected_derive
                .replace("#[", "# [")
                .replace('(', " (")
                .replace(')', " )"), // # [derive (Debug )]
            expected_derive.replace("#[", "# [").replace('(', " ("), // # [derive (Debug)]
        ];

        let contains_derive = variations.iter().any(|v| result_str.contains(v));
        assert!(
            contains_derive,
            "Should contain derive attribute for trait {trait_name}\nExpected one of: {variations:?}\nActual output:\n{result_str}"
        );
    }
}

#[test]
fn test_derive_ast_structure() {
    let code = r"
        #[derive(Debug, Clone)]
        class Test {
            field: i32
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    // Verify AST structure contains derive information
    if let ruchy::frontend::ast::ExprKind::Class { derives, .. } = &ast.kind {
        assert_eq!(derives.len(), 2, "Should have 2 derives");
        assert_eq!(derives[0], "Debug", "First derive should be Debug");
        assert_eq!(derives[1], "Clone", "Second derive should be Clone");
    } else {
        panic!("AST should be a class");
    }
}

#[test]
fn test_class_without_derives() {
    let code = r"
        class Simple {
            value: i32
        }
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");

    let transpiler = Transpiler::new();
    let result = transpiler
        .transpile(&ast)
        .expect("Should transpile successfully");
    let result_str = result.to_string();

    // Should not have any derive attributes
    assert!(!result_str.contains("#[derive"));
    assert!(result_str.contains("struct Simple"));
}

// TODO: Implement validation for empty derive lists
// #[test]
// fn test_derive_parsing_failure_cases() {
//     // Test invalid derive syntax
//     let code1 = r#"
//         #[derive()]
//         class Test {
//             field: i32
//         }
//     "#;

//     let mut parser1 = Parser::new(code1);
//     let result1 = parser1.parse();
//     assert!(result1.is_err(), "Should fail with empty derive list");

//     // Test invalid derive syntax - missing closing parenthesis
//     let code2 = r#"
//         #[derive(Debug
//         class Test {
//             field: i32
//         }
//     "#;

//     let mut parser2 = Parser::new(code2);
//     let result2 = parser2.parse();
//     assert!(result2.is_err(), "Should fail with malformed derive");
// }

// TODO: Implement support for preserving non-derive attributes
// #[test]
// fn test_nested_attributes() {
//     let code = r#"
//         #[derive(Debug)]
//         #[serde(rename_all = "camelCase")]
//         class ApiResponse {
//             status: String,
//             data: Vec<i32>
//         }
//     "#;

//     let mut parser = Parser::new(code);
//     let ast = parser.parse().expect("Should parse successfully");

//     let transpiler = Transpiler::new();
//     let result = transpiler.transpile(&ast).expect("Should transpile successfully");
//     let result_str = result.to_string();

//     // Should preserve both attributes
//     assert!(result_str.contains("# [derive (Debug)]")
//             || result_str.contains("#[derive(Debug)]"));
//     assert!(result_str.contains("#[serde(rename_all = \"camelCase\")]"));
// }
