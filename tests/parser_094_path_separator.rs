//! PARSER-094: Fix Module Path Separator Bug (:: → .)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Fix transpiler converting :: to . (Issue #137)
//! Target: Preserve Rust path separator :: in transpiled code
//!
//! Root Cause: Transpiler treats :: as `FieldAccess` instead of `PathSeparator`
//! Impact: Breaks all module function calls (`http_client::http_get` becomes `http_client.http_get`)
//!
//! Test Strategy:
//! 1. Module function calls - `http_client::http_get()`
//! 2. Stdlib qualified paths - `std::io::Read`
//! 3. Nested module paths - `nested::module::function`
//! 4. Type associated functions - `String::from()`
//! 5. Integration - Full examples from Issue #137

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

// ============================================================================
// RED-001: Module Function Calls
// ============================================================================

#[test]
fn test_parser_094_module_function_call_simple() {
    // Test: http_client::http_get() should preserve ::
    let code = r"
        let result = http_client::http_get();
        result
    ";

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("http_client :: http_get"),
        "Should preserve :: in module function call, got: {rust_code}"
    );
    assert!(
        !rust_code.contains("http_client . http_get"),
        "Should NOT convert :: to ., got: {rust_code}"
    );
}

#[test]
fn test_parser_094_module_function_with_args() {
    // Test: Module function call with arguments
    let code = r#"
        let result = http_client::http_get("/api/data", "token");
        result
    "#;

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("http_client :: http_get"),
        "Should preserve :: with arguments, got: {rust_code}"
    );
}

#[test]
fn test_parser_094_nested_module_path() {
    // Test: http_client::helpers::get_json (realistic nested module path with underscores)
    // Issue #137: Real-world pattern from ruchy-lambda
    let code = r"
        let result = http_client::helpers::get_json();
        result
    ";

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("http_client :: helpers :: get_json"),
        "Should preserve :: in nested module paths, got: {rust_code}"
    );
}

// ============================================================================
// RED-002: Stdlib Qualified Paths
// ============================================================================

#[test]
fn test_parser_094_stdlib_path() {
    // Test: std::io::Read qualified path
    let code = r"
        let stream = std::io::stdin();
        stream
    ";

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("std :: io :: stdin"),
        "Should preserve :: in stdlib paths, got: {rust_code}"
    );
}

#[test]
fn test_parser_094_multiple_stdlib_calls() {
    // Test: Multiple stdlib calls with ::
    let code = r#"
        let env_var = std::env::var("PATH");
        let home = std::env::home_dir();
        env_var
    "#;

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("std :: env :: var"),
        "Should preserve first :: call, got: {rust_code}"
    );
    assert!(
        rust_code.contains("std :: env :: home_dir"),
        "Should preserve second :: call, got: {rust_code}"
    );
}

// ============================================================================
// RED-003: Type Associated Functions
// ============================================================================

#[test]
fn test_parser_094_type_associated_function() {
    // Test: String::from() associated function
    let code = r#"
        let s = String::from("hello");
        s
    "#;

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("String :: from"),
        "Should preserve :: in associated function, got: {rust_code}"
    );
}

#[test]
fn test_parser_094_vec_new() {
    // Test: Vec::new() associated function
    let code = r"
        let v = Vec::new();
        v
    ";

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("Vec :: new"),
        "Should preserve :: in Vec::new, got: {rust_code}"
    );
}

// ============================================================================
// RED-004: Issue #137 Reproduction
// ============================================================================

#[test]
fn test_parser_094_issue_137_reproduction() {
    // Test: Exact pattern from Issue #137
    let code = r#"
        let endpoint = "127.0.0.1:8080";
        let path = "/api/invoke";
        let result = http_client::http_get(&endpoint, &path);
        result
    "#;

    let ast = Parser::new(code)
        .parse()
        .expect("Should parse Issue #137 code");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    assert!(
        rust_code.contains("http_client :: http_get"),
        "Issue #137: Should preserve :: in module call, got: {rust_code}"
    );
    assert!(
        !rust_code.contains("http_client . http_get"),
        "Issue #137: Should NOT have . separator, got: {rust_code}"
    );
}

// ============================================================================
// RED-005: Mixed :: and . (Field Access vs Path)
// ============================================================================

#[test]
fn test_parser_094_distinguish_field_access_from_path() {
    // Test: Should distinguish obj.field from Module::function
    let code = r"
        let obj = MyStruct { field: 42 };
        let field_value = obj.field;
        let module_result = MyModule::function();
        field_value
    ";

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    // Field access should use .
    assert!(
        rust_code.contains("obj . field"),
        "Field access should use ., got: {rust_code}"
    );

    // Module path should use ::
    assert!(
        rust_code.contains("MyModule :: function"),
        "Module path should use ::, got: {rust_code}"
    );
}

#[test]
fn test_parser_094_method_call_vs_static_call() {
    // Test: instance.method() vs Type::function()
    let code = r#"
        let s = String::from("test");
        let len = s.len();
        len
    "#;

    let ast = Parser::new(code).parse().expect("Should parse");
    let rust_code = Transpiler::new()
        .transpile(&ast)
        .expect("Should transpile")
        .to_string();

    // Associated function should use ::
    assert!(
        rust_code.contains("String :: from"),
        "Associated function should use ::, got: {rust_code}"
    );

    // Method call should use .
    assert!(
        rust_code.contains("s . len ()"),
        "Method call should use ., got: {rust_code}"
    );
}
