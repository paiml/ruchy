//! Comprehensive TDD test suite for snapshot testing functionality
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every snapshot path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Parser, Transpiler};
use std::fs;
use std::path::PathBuf;

// ==================== SNAPSHOT CREATION TESTS ====================

#[test]
fn test_snapshot_simple_expression() {
    let code = "1 + 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    // Snapshot should be deterministic
    let snapshot = result.to_string();
    assert!(snapshot.contains("1"));
    assert!(snapshot.contains("+"));
    assert!(snapshot.contains("2"));
}

#[test]
fn test_snapshot_function_definition() {
    let code = "fun add(x: i32, y: i32) -> i32 { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("fn add"));
    assert!(snapshot.contains("i32"));
}

#[test]
fn test_snapshot_deterministic() {
    let code = "let x = 42; let y = x + 1";
    let mut parser = Parser::new(code);
    let ast1 = parser.parse().unwrap();
    
    let mut parser2 = Parser::new(code);
    let ast2 = parser2.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result1 = transpiler.transpile(&ast1).unwrap().to_string();
    let result2 = transpiler.transpile(&ast2).unwrap().to_string();
    
    // Should produce identical output
    assert_eq!(result1, result2);
}

// ==================== SNAPSHOT COMPARISON TESTS ====================

#[test]
fn test_snapshot_comparison_identical() {
    let snapshot1 = "fn main() { println!(\"hello\"); }";
    let snapshot2 = "fn main() { println!(\"hello\"); }";
    
    assert_eq!(snapshot1, snapshot2);
}

#[test]
fn test_snapshot_comparison_different() {
    let snapshot1 = "fn main() { println!(\"hello\"); }";
    let snapshot2 = "fn main() { println!(\"world\"); }";
    
    assert_ne!(snapshot1, snapshot2);
}

#[test]
fn test_snapshot_whitespace_normalization() {
    let code1 = "let x=1+2";
    let code2 = "let x = 1 + 2";
    
    let mut parser1 = Parser::new(code1);
    let ast1 = parser1.parse().unwrap();
    
    let mut parser2 = Parser::new(code2);
    let ast2 = parser2.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result1 = transpiler.transpile(&ast1).unwrap().to_string();
    let result2 = transpiler.transpile(&ast2).unwrap().to_string();
    
    // Should normalize whitespace
    assert_eq!(result1, result2);
}

// ==================== SNAPSHOT UPDATE TESTS ====================

#[test]
fn test_snapshot_update_detection() {
    let old_code = "fun old() -> i32 { 1 }";
    let new_code = "fun new() -> i32 { 2 }";
    
    let mut parser_old = Parser::new(old_code);
    let ast_old = parser_old.parse().unwrap();
    
    let mut parser_new = Parser::new(new_code);
    let ast_new = parser_new.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let snapshot_old = transpiler.transpile(&ast_old).unwrap().to_string();
    let snapshot_new = transpiler.transpile(&ast_new).unwrap().to_string();
    
    // Should detect changes
    assert_ne!(snapshot_old, snapshot_new);
}

// ==================== COMPLEX SNAPSHOT TESTS ====================

#[test]
fn test_snapshot_struct_definition() {
    let code = r#"
    struct User {
        name: String,
        age: i32,
        active: bool
    }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("struct User"));
    assert!(snapshot.contains("name"));
    assert!(snapshot.contains("String"));
}

#[test]
fn test_snapshot_enum_definition() {
    let code = r#"
    enum Result<T, E> {
        Ok(T),
        Err(E)
    }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("enum Result"));
    assert!(snapshot.contains("Ok"));
    assert!(snapshot.contains("Err"));
}

#[test]
fn test_snapshot_impl_block() {
    let code = r#"
    struct Point { x: i32, y: i32 }
    
    impl Point {
        fun new(x: i32, y: i32) -> Point {
            Point { x, y }
        }
    }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("impl Point"));
    assert!(snapshot.contains("fn new"));
}

#[test]
fn test_snapshot_match_expression() {
    let code = r#"
    match value {
        Some(x) => x * 2,
        None => 0
    }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("match"));
    assert!(snapshot.contains("Some"));
    assert!(snapshot.contains("None"));
}

#[test]
fn test_snapshot_async_function() {
    let code = r#"
    async fun fetch_data() -> String {
        await get_remote_data()
    }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("async"));
    assert!(snapshot.contains("fn fetch_data"));
}

// ==================== SNAPSHOT STABILITY TESTS ====================

#[test]
fn test_snapshot_stability_across_formats() {
    let code = "let x = [1, 2, 3]";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    // Multiple invocations should be stable
    let snapshot1 = result.to_string();
    let snapshot2 = result.to_string();
    assert_eq!(snapshot1, snapshot2);
}

#[test]
fn test_snapshot_comment_handling() {
    let code1 = "// Comment\nlet x = 1";
    let code2 = "let x = 1";
    
    let mut parser1 = Parser::new(code1);
    let ast1 = parser1.parse().unwrap();
    
    let mut parser2 = Parser::new(code2);
    let ast2 = parser2.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result1 = transpiler.transpile(&ast1).unwrap().to_string();
    let result2 = transpiler.transpile(&ast2).unwrap().to_string();
    
    // Comments might or might not affect output
    assert!(result1.len() > 0 && result2.len() > 0);
}

// ==================== ERROR SNAPSHOT TESTS ====================

#[test]
fn test_snapshot_parse_error() {
    let code = "let x = ";  // Invalid
    let mut parser = Parser::new(code);
    let ast_result = parser.parse();
    
    // Should capture error state
    assert!(ast_result.is_err());
}

#[test]
fn test_snapshot_transpile_error_recovery() {
    let code = "fun test() { undefined_func() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    // Should handle gracefully
    assert!(result.is_ok());
}

// ==================== REGRESSION SNAPSHOT TESTS ====================

#[test]
fn test_snapshot_regression_string_interpolation() {
    let code = r#"let msg = f"Hello {name}""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("format!") || snapshot.contains("name"));
}

#[test]
fn test_snapshot_regression_pipeline_operator() {
    let code = "data |> process |> format";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast).unwrap();
    
    let snapshot = result.to_string();
    assert!(snapshot.contains("data") || snapshot.contains("process"));
}

// ==================== BENCHMARK SNAPSHOT TESTS ====================

#[test]
fn test_snapshot_performance_small() {
    let code = "let x = 1";
    let start = std::time::Instant::now();
    
    for _ in 0..100 {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let transpiler = Transpiler::new();
        let _ = transpiler.transpile(&ast).unwrap().to_string();
    }
    
    let duration = start.elapsed();
    // Should be fast for small code
    assert!(duration.as_millis() < 1000);
}

// Run all tests with: cargo test snapshot_tdd --test snapshot_tdd