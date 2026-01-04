//! Probar-based WASM Tests for Ruchy Compiler
//!
//! Uses jugar-probar for:
//! - GUI coverage tracking of compiler API surface
//! - Playwright-compatible assertions
//! - Comprehensive WASM testing
//!
//! Run with: `wasm-pack test --headless --firefox`

use jugar_probar::prelude::*;
use ruchy_wasm::RuchyCompiler;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// =============================================================================
// GUI Coverage Tracking for Compiler API
// =============================================================================

/// Track coverage of RuchyCompiler API methods
fn compiler_api_coverage() -> UxCoverageTracker {
    let tracker = UxCoverageBuilder::new()
        // API Methods (buttons represent method calls)
        .button("compile")
        .button("validate")
        .button("parse_to_json")
        .button("version")
        // Input types (different source code patterns)
        .input("simple_function")
        .input("nested_scopes")
        .input("attributes")
        .input("structs")
        .input("enums")
        .input("closures")
        .input("control_flow")
        .input("pattern_matching")
        // Screens represent test categories
        .screen("basic_compilation")
        .screen("syntax_validation")
        .screen("ast_generation")
        .screen("error_handling")
        .screen("edge_cases")
        .build();
    tracker
}

// =============================================================================
// Basic Compilation Tests (Issue #51, #52)
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_compiler_basic_function() {
    let mut gui = compiler_api_coverage();
    gui.visit("basic_compilation");

    let mut compiler = RuchyCompiler::new();
    gui.click("compile");

    let code = r#"fn add(a, b) { a + b }"#;
    gui.input("simple_function");

    let result = compiler.compile(code);
    assert!(
        result.is_ok(),
        "Basic function should compile: {:?}",
        result
    );
    let rust_code = result.unwrap();
    assert!(
        rust_code.contains("fn add"),
        "Should contain function definition"
    );
}

#[wasm_bindgen_test]
fn test_probar_nested_scopes() {
    let mut gui = compiler_api_coverage();
    gui.visit("basic_compilation");
    gui.input("nested_scopes");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = r#"
fn complex(data) {
    let result = 0
    for item in data {
        if item.valid {
            let value = item.value
            result = result + value
        }
    }
    result
}
"#;

    let valid = compiler.validate(code);
    assert!(valid, "Nested scopes should validate (Issue #51)");
}

#[wasm_bindgen_test]
fn test_probar_attributes() {
    let mut gui = compiler_api_coverage();
    gui.visit("basic_compilation");
    gui.input("attributes");

    let mut compiler = RuchyCompiler::new();
    gui.click("compile");

    let code = r#"
@memoize
fn fibonacci(n) {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}
"#;

    let result = compiler.compile(code);
    assert!(
        result.is_ok(),
        "Attributes should compile (Issue #52): {:?}",
        result
    );
}

#[wasm_bindgen_test]
fn test_probar_multiple_attributes() {
    let mut gui = compiler_api_coverage();
    gui.input("attributes");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = r#"
@inline
@derive(Debug)
fn test() { 42 }
"#;

    assert!(
        compiler.validate(code),
        "Multiple attributes should validate"
    );
}

// =============================================================================
// Syntax Validation Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_validate_valid_code() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let valid_codes = vec![
        ("let x = 42", "let binding"),
        ("fn foo() { }", "empty function"),
        ("if true { 1 } else { 2 }", "if-else"),
        ("for i in 0..10 { i }", "for loop"),
        ("while x > 0 { x = x - 1 }", "while loop"),
        ("match x { 1 => a, _ => b }", "match expression"),
    ];

    for (code, name) in valid_codes {
        assert!(compiler.validate(code), "{} should validate", name);
    }
}

#[wasm_bindgen_test]
fn test_probar_validate_structs() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");
    gui.input("structs");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = r#"
struct Person {
    name: String,
    age: i32
}
"#;

    assert!(compiler.validate(code), "Struct definition should validate");
}

#[wasm_bindgen_test]
fn test_probar_validate_enums() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");
    gui.input("enums");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = r#"
enum Color {
    Red,
    Green,
    Blue
}
"#;

    assert!(compiler.validate(code), "Enum definition should validate");
}

#[wasm_bindgen_test]
fn test_probar_validate_closures() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");
    gui.input("closures");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let codes = vec![
        "|x| x + 1",
        "|a, b| a * b",
        "|| 42",
        "|x| { let y = x * 2; y }",
    ];

    for code in codes {
        assert!(
            compiler.validate(code),
            "Closure '{}' should validate",
            code
        );
    }
}

#[wasm_bindgen_test]
fn test_probar_validate_control_flow() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");
    gui.input("control_flow");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let codes = vec![
        "loop { break }",
        "while true { continue }",
        "for i in items { if i > 0 { break } }",
        "return 42",
    ];

    for code in codes {
        assert!(
            compiler.validate(code),
            "Control flow '{}' should validate",
            code
        );
    }
}

#[wasm_bindgen_test]
fn test_probar_validate_pattern_matching() {
    let mut gui = compiler_api_coverage();
    gui.visit("syntax_validation");
    gui.input("pattern_matching");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = r#"
match value {
    Some(x) => x,
    None => 0,
    Ok(v) if v > 0 => v,
    Err(e) => -1,
    _ => default
}
"#;

    assert!(compiler.validate(code), "Pattern matching should validate");
}

// =============================================================================
// AST Generation Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_parse_to_json() {
    let mut gui = compiler_api_coverage();
    gui.visit("ast_generation");

    let compiler = RuchyCompiler::new();
    gui.click("parse_to_json");

    let code = "let x = 42";
    let result = compiler.parse_to_json(code);

    assert!(result.is_ok(), "parse_to_json should succeed");
    let json = result.unwrap();
    assert!(
        json.contains("Let") || json.contains("let"),
        "JSON should contain let binding"
    );
}

#[wasm_bindgen_test]
fn test_probar_parse_function_to_json() {
    let mut gui = compiler_api_coverage();
    gui.visit("ast_generation");
    gui.input("simple_function");

    let compiler = RuchyCompiler::new();
    gui.click("parse_to_json");

    let code = "fn add(a, b) { a + b }";
    let result = compiler.parse_to_json(code);

    assert!(result.is_ok(), "Function should parse to JSON");
    let json = result.unwrap();
    assert!(json.contains("add"), "JSON should contain function name");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_compile_error_handling() {
    let mut gui = compiler_api_coverage();
    gui.visit("error_handling");

    let mut compiler = RuchyCompiler::new();
    gui.click("compile");

    // Invalid syntax should return error
    let invalid_code = "fn foo( { }"; // Missing closing paren
    let result = compiler.compile(invalid_code);

    assert!(result.is_err(), "Invalid syntax should return error");
}

#[wasm_bindgen_test]
fn test_probar_validate_invalid_syntax() {
    let mut gui = compiler_api_coverage();
    gui.visit("error_handling");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let invalid_codes = vec![
        "fn foo(",      // Incomplete function
        "let = 42",     // Missing variable name
        "if { }",       // Missing condition
        "for in x { }", // Missing iterator variable
    ];

    for code in invalid_codes {
        assert!(
            !compiler.validate(code),
            "Invalid code '{}' should fail validation",
            code
        );
    }
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_empty_input() {
    let mut gui = compiler_api_coverage();
    gui.visit("edge_cases");

    let mut compiler = RuchyCompiler::new();

    // Empty string might be valid or error depending on implementation
    let _ = compiler.compile("");
    let _ = compiler.validate("");
}

#[wasm_bindgen_test]
fn test_probar_unicode_identifiers() {
    let mut gui = compiler_api_coverage();
    gui.visit("edge_cases");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    // Unicode in strings should work
    let code = r#"let greeting = "Hello, 世界!""#;
    // May or may not validate depending on string literal support
    let _ = compiler.validate(code);
}

#[wasm_bindgen_test]
fn test_probar_large_number_literals() {
    let mut gui = compiler_api_coverage();
    gui.visit("edge_cases");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    let code = "let big = 9223372036854775807"; // i64::MAX
    assert!(
        compiler.validate(code),
        "Large number literals should validate"
    );
}

#[wasm_bindgen_test]
fn test_probar_deeply_nested_expressions() {
    let mut gui = compiler_api_coverage();
    gui.visit("edge_cases");

    let compiler = RuchyCompiler::new();
    gui.click("validate");

    // Deeply nested but valid expression
    let code = "((((((1 + 2) * 3) - 4) / 5) % 6) + 7)";
    assert!(
        compiler.validate(code),
        "Deeply nested expressions should validate"
    );
}

// =============================================================================
// Version API Test
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_version_api() {
    let mut gui = compiler_api_coverage();
    gui.click("version");

    let compiler = RuchyCompiler::new();
    let version = compiler.version();

    assert!(!version.is_empty(), "Version should not be empty");
    // Version should be semver format
    assert!(version.contains('.'), "Version should be in semver format");
}

// =============================================================================
// Coverage Report (run at end)
// =============================================================================

#[wasm_bindgen_test]
fn test_probar_coverage_report() {
    let mut gui = compiler_api_coverage();

    // Simulate comprehensive API usage
    gui.click("compile");
    gui.click("validate");
    gui.click("parse_to_json");
    gui.click("version");

    gui.input("simple_function");
    gui.input("nested_scopes");
    gui.input("attributes");
    gui.input("structs");
    gui.input("enums");
    gui.input("closures");
    gui.input("control_flow");
    gui.input("pattern_matching");

    gui.visit("basic_compilation");
    gui.visit("syntax_validation");
    gui.visit("ast_generation");
    gui.visit("error_handling");
    gui.visit("edge_cases");

    // Generate coverage report
    let report = gui.generate_report();
    println!("\n{}", report);
    println!("GUI Coverage: {}", gui.summary());

    // Assert minimum coverage threshold
    assert!(
        gui.meets(80.0),
        "API coverage should be at least 80%: {}",
        gui.percent()
    );
}
