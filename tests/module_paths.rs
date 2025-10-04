#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Tests for module path (::) syntax support
#![allow(clippy::print_stdout, clippy::uninlined_format_args)] // Test debugging output

use ruchy::frontend::ast::{ExprKind, TypeKind};
use ruchy::frontend::parser::Parser;

#[test]
fn test_qualified_type_names() {
    let cases = vec![
        "fn test(x: std::string::String) { x }",
        "fn test(x: std::io::Result) { x }",
        "fn test(x: std::collections::HashMap) { x }",
        "fn test(x: a::b::c::d::e::VeryLongType) { x }",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);

        if let Ok(expr) = result {
            if let ExprKind::Function { params, .. } = &expr.kind {
                assert_eq!(params.len(), 1);
                let param_type = &params[0].ty;
                if let TypeKind::Named(name) = &param_type.kind {
                    assert!(name.contains("::"), "Type should contain :: - got {}", name);
                }
            }
        }
    }
}

#[test]
fn test_qualified_generic_types() {
    let cases = vec![
        "fn test(x: std::result::Result<String, Error>) { x }",
        "fn test(x: std::collections::HashMap<String, i32>) { x }",
        "fn test(x: std::vec::Vec<T>) { x }",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
    }
}

#[test]
fn test_qualified_function_calls() {
    let cases = vec![
        "std::fs::read_file(\"test.txt\")",
        "std::io::println(\"hello\")",
        "MyModule::my_function(42)",
        "a::b::c::deeply::nested::function()",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
    }
}

#[test]
fn test_qualified_in_expressions() {
    let cases = vec![
        "let x = std::collections::HashMap::new()",
        "let result = std::fs::read_file(path)",
        "if std::env::var(\"DEBUG\").is_ok() { true } else { false }",
        "match std::fs::metadata(path) { Ok(m) => m, Err(e) => panic!(e) }",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
    }
}

#[test]
fn test_result_option_qualified() {
    let cases = vec![
        "Result::Ok(42)",
        "Result::Err(\"error\")",
        "Option::Some(42)",
        "Option::None",
        "std::result::Result::Ok(42)",
        "std::option::Option::Some(42)",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", input);
    }
}

#[test]
fn test_use_statements() {
    let cases = vec![
        "use std::collections::HashMap",
        "use std::fs::{File, OpenOptions}",
        "use std::io::*",
        "use std::collections::HashMap as Map",
    ];

    for input in cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse use statement: {}", input);
    }
}

#[test]
fn test_three_segment_qualified_call() {
    // This should replicate the REPL failure case exactly
    let input = "std::fs::read_file(\"test.txt\")";
    let mut parser = Parser::new(input);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Failed to parse: {} - Error: {:?}",
        input,
        result.err()
    );
}

#[test]
fn debug_repl_vs_unittest() {
    // TOYOTA WAY: Go to the source - debug the exact difference
    let input = "std::fs::read_file(\"test.txt\")";

    println!("=== UNIT TEST CONTEXT DEBUG ===");
    let mut parser = Parser::new(input);
    let result = parser.parse();
    println!("Unit test result: {:?}", result.is_ok());

    if let Err(e) = &result {
        println!("Unit test error: {:?}", e);
        // Print the error chain for full context
        let mut current = e.source();
        while let Some(cause) = current {
            println!("  Caused by: {:?}", cause);
            current = cause.source();
        }
    }

    // This test MUST pass - if it fails, something is fundamentally wrong
    assert!(
        result.is_ok(),
        "Unit test context MUST work for qualified calls"
    );
}
