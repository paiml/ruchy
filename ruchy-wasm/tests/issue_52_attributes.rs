//! Test for GitHub Issue #52: WASM attributes (@) syntax parsing
//!
//! Bug: Attributes work in native Ruchy but fail in WASM with
//! "Unexpected token: AttributeStart" errors.
//!
//! This test verifies that attribute syntax parses correctly in WASM target.

use ruchy_wasm::RuchyCompiler;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_issue_52_simple_attribute() {
    let mut compiler = RuchyCompiler::new();

    let code = r#"
@memoize
fn fibonacci(n) {
  if n <= 1 {
    n
  } else {
    fibonacci(n - 1) + fibonacci(n - 2)
  }
}
"#;

    let result = compiler.compile(code);

    // Should compile successfully, not fail with "Unexpected token: AttributeStart"
    match result {
        Ok(rust_code) => {
            assert!(rust_code.contains("fibonacci"));
            println!("✅ Attribute parsing succeeded: {}", rust_code);
        }
        Err(e) => {
            let err_msg = format!("{:?}", e);
            panic!("❌ Attribute parsing failed: {}", err_msg);
        }
    }
}

#[wasm_bindgen_test]
fn test_issue_52_multiple_attributes() {
    let mut compiler = RuchyCompiler::new();

    let code = r#"
@memoize
@inline
fn test() {
    42
}
"#;

    let result = compiler.validate(code);
    assert!(result, "Multiple attributes should parse successfully");
}

#[wasm_bindgen_test]
fn test_issue_52_attribute_with_args() {
    let mut compiler = RuchyCompiler::new();

    let code = r#"
@derive(Debug, Clone)
struct Person {
    name: String,
    age: i32
}
"#;

    let result = compiler.validate(code);
    assert!(
        result,
        "Attributes with arguments should parse successfully"
    );
}
