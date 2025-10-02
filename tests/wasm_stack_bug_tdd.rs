/// [WASM-001] TDD tests for WASM stack management bugs
///
/// Testing the exact failure cases from Issue #27
use ruchy::backend::wasm::WasmEmitter;
use ruchy::frontend::Parser;

fn compile_to_wasm(code: &str) -> Result<Vec<u8>, String> {
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let emitter = WasmEmitter::new();
    emitter.emit(&expr)
}

fn validate_wasm(bytes: &[u8]) -> Result<(), String> {
    use wasmparser::{Validator, WasmFeatures};

    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(bytes)
        .map_err(|e| format!("WASM validation failed: {}", e))?;
    Ok(())
}

#[test]
fn test_issue_27_example_1_arithmetic_stack_overflow() {
    // From Issue #27 Example 1: Stack overflow - values remaining
    let code = r#"
2 + 2
10 * 5
100 - 25
50 / 2
17 % 5
    "#;

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    // This should FAIL with "values remaining on stack at end of block"
    let result = validate_wasm(&bytes);

    if let Err(e) = &result {
        println!("Expected failure: {}", e);
        assert!(
            e.contains("values remaining on stack") || e.contains("type mismatch"),
            "Should fail with stack/type error, got: {}",
            e
        );
    } else {
        panic!("WASM validation should fail for multi-expression code without proper drops");
    }
}

#[test]
fn test_issue_27_example_2_type_inference() {
    // From Issue #27 Example 2: Type inference error (i32 vs f32)
    let code = r#"
let x = 10
let y = 20
x + y
let pi = 3.14159
let radius = 5
let area = pi * radius * radius
area
    "#;

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    // This should FAIL with type mismatch (i32 vs f32)
    let result = validate_wasm(&bytes);

    if let Err(e) = &result {
        println!("Expected failure: {}", e);
        assert!(
            e.contains("type mismatch"),
            "Should fail with type mismatch, got: {}",
            e
        );
    } else {
        panic!("WASM validation should fail for mixed int/float operations");
    }
}

#[test]
fn test_issue_27_working_example() {
    // From Issue #27: Only trivial code works
    let code = r#"
fun add(a, b) {
    a + b
}
add(2, 3)
    "#;

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    // This SHOULD work according to the issue
    validate_wasm(&bytes).expect("Trivial single-function-call should validate successfully");

    println!("✓ Trivial code validates successfully (as expected)");
}

#[test]
fn test_simple_two_expressions() {
    // Minimal reproduction case
    let code = "2 + 2\n10 * 5";

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    let result = validate_wasm(&bytes);

    if let Err(e) = &result {
        println!("Minimal case failure: {}", e);
        assert!(
            e.contains("values remaining") || e.contains("type mismatch"),
            "Should fail with stack error, got: {}",
            e
        );
    } else {
        panic!("Even two expressions should fail validation without proper stack cleanup");
    }
}

#[test]
fn test_single_expression_works() {
    // This SHOULD work - single expression
    let code = "2 + 2";

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    validate_wasm(&bytes).expect("Single expression should validate");

    println!("✓ Single expression validates (expected behavior)");
}
