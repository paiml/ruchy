//! Test for GitHub Issue #51: WASM multi-line code blocks with nested scopes
//!
//! Bug: Multi-line blocks with nested scopes (let, if, for inside functions/loops)
//! fail to parse in WASM with "Expected RightBrace" errors.
//!
//! These same blocks work correctly in native Ruchy CLI.

use ruchy_wasm::RuchyCompiler;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_issue_51_function_with_nested_let() {
    let compiler = RuchyCompiler::new();

    let code = r#"
fn calculate_total(items) {
  let sum = 0
  for item in items {
    let price = item.price
    sum = sum + price
  }
  sum
}
"#;

    let result = compiler.validate(code);
    assert!(result, "Function with nested let should parse successfully");
}

#[wasm_bindgen_test]
fn test_issue_51_if_block_with_nested_code() {
    let compiler = RuchyCompiler::new();

    let code = r#"
fn check_adult(age) {
  if age >= 18 {
    let status = "adult"
    status
  }
}
"#;

    let result = compiler.validate(code);
    assert!(result, "If block with nested let should parse successfully");
}

#[wasm_bindgen_test]
fn test_issue_51_loop_with_nested_if() {
    let compiler = RuchyCompiler::new();

    let code = r#"
fn process_items(items) {
  for item in items {
    if item.active {
      process(item)
    }
  }
}
"#;

    let result = compiler.validate(code);
    assert!(result, "Loop with nested if should parse successfully");
}

#[wasm_bindgen_test]
fn test_issue_51_deeply_nested_scopes() {
    let compiler = RuchyCompiler::new();

    let code = r#"
fn complex_logic(data) {
  let result = 0
  for item in data {
    if item.valid {
      let value = item.value
      if value > 0 {
        result = result + value
      }
    }
  }
  result
}
"#;

    let result = compiler.validate(code);
    assert!(result, "Deeply nested scopes should parse successfully");
}

#[wasm_bindgen_test]
fn test_issue_51_multiple_lets_in_function() {
    let compiler = RuchyCompiler::new();

    let code = r#"
fn test() {
  let a = 1
  let b = 2
  let c = 3
  a + b + c
}
"#;

    let result = compiler.validate(code);
    assert!(result, "Multiple let statements should parse successfully");
}
