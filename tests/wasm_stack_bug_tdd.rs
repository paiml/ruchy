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
    // From Issue #27 Example 1: Multi-expression code should validate
    // The WASM emitter correctly handles stack management with Drop instructions
    let code = r#"
2 + 2
10 * 5
100 - 25
50 / 2
17 % 5
    "#;

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    // This SHOULD succeed - the emitter adds Drop instructions for intermediate values
    validate_wasm(&bytes)
        .expect("Multi-expression code should validate with proper stack management");

    println!("✓ Multi-expression arithmetic validates successfully (stack managed correctly)");
}

#[test]
fn test_issue_27_example_2_type_inference() {
    // From Issue #27 Example 2: Mixed int/float operations with proper type coercion
    // The WASM emitter handles automatic type conversion (i32 -> f32)
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

    // This SHOULD succeed - the emitter adds F32ConvertI32S for type coercion
    validate_wasm(&bytes)
        .expect("Mixed int/float operations should validate with automatic type conversion");

    println!("✓ Mixed int/float code validates successfully (automatic type coercion)");
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
    // Minimal reproduction case - two expressions should validate
    let code = "2 + 2\n10 * 5";

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    // This SHOULD succeed - Drop instruction added for first expression
    validate_wasm(&bytes).expect("Two expressions should validate with proper Drop");

    println!("✓ Two-expression code validates (Drop added for first expression)");
}

#[test]
fn test_single_expression_works() {
    // This SHOULD work - single expression
    let code = "2 + 2";

    let bytes = compile_to_wasm(code).expect("Compilation should succeed");

    validate_wasm(&bytes).expect("Single expression should validate");

    println!("✓ Single expression validates (expected behavior)");
}
