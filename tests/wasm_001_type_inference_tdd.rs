/// [WASM-001] TDD tests for type-aware WASM code generation
///
/// Root cause: Binary operations hardcoded to i32, causing type mismatches
/// when mixing integer and float operations.
///
/// Solution: Implement type inference to choose correct WASM instructions
/// (I32Add vs F32Add, I32Mul vs F32Mul, etc.)
use ruchy::backend::wasm::WasmEmitter;
use ruchy::frontend::Parser;
use wasmparser::{Validator, WasmFeatures};

fn compile_and_validate(code: &str) -> Result<Vec<u8>, String> {
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let emitter = WasmEmitter::new();
    let bytes = emitter.emit(&expr)?;

    // Validate WASM
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    validator
        .validate_all(&bytes)
        .map_err(|e| format!("WASM validation failed: {}", e))?;

    Ok(bytes)
}

// ===== INTEGER OPERATIONS (Baseline - Should Work) =====

#[test]
fn test_pure_integer_operations() {
    // Pure integer arithmetic should work (baseline)
    let code = "2 + 3";
    compile_and_validate(code).expect("Pure integer operations should validate");
}

#[test]
fn test_integer_multiplication() {
    let code = "10 * 5";
    compile_and_validate(code).expect("Integer multiplication should validate");
}

#[test]
fn test_integer_variables() {
    let code = r#"
        let x = 10
        let y = 20
        x + y
    "#;
    compile_and_validate(code).expect("Integer variables should validate");
}

// ===== FLOAT OPERATIONS (Currently Broken) =====

#[test]
#[ignore] // Enable when type inference implemented
fn test_pure_float_operations() {
    let code = "3.14 + 2.71";
    compile_and_validate(code).expect("Pure float operations should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_multiplication() {
    let code = "3.14 * 2.0";
    compile_and_validate(code).expect("Float multiplication should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_variables() {
    let code = r#"
        let pi = 3.14159
        let radius = 2.5
        pi * radius
    "#;
    compile_and_validate(code).expect("Float variables should validate");
}

// ===== MIXED INT/FLOAT OPERATIONS (Currently Broken) =====

#[test]
#[ignore] // Enable when type inference implemented
fn test_mixed_int_float_multiplication() {
    // This is the EXACT failure case from Issue #27
    let code = r#"
        let pi = 3.14159
        let radius = 5
        pi * radius * radius
    "#;
    compile_and_validate(code).expect("Mixed int/float should validate with type promotion");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_times_int() {
    let code = "3.14 * 10";
    compile_and_validate(code).expect("Float * int should validate (promote to f32)");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_int_times_float() {
    let code = "10 * 3.14";
    compile_and_validate(code).expect("Int * float should validate (promote to f32)");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_mixed_addition() {
    let code = "10 + 3.14";
    compile_and_validate(code).expect("Int + float should validate (promote to f32)");
}

// ===== COMPLEX EXPRESSIONS =====

#[test]
#[ignore] // Enable when type inference implemented
fn test_area_calculation() {
    let code = r#"
        let pi = 3.14159
        let radius = 5
        let area = pi * radius * radius
        area
    "#;
    compile_and_validate(code).expect("Area calculation should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_nested_mixed_operations() {
    let code = "(3.14 + 1.0) * (10 + 5)";
    compile_and_validate(code).expect("Nested mixed operations should validate");
}

// ===== DIVISION AND MODULO =====

#[test]
fn test_integer_division() {
    let code = "10 / 2";
    compile_and_validate(code).expect("Integer division should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_division() {
    let code = "10.0 / 3.0";
    compile_and_validate(code).expect("Float division should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_mixed_division() {
    let code = "10.0 / 2";
    compile_and_validate(code).expect("Mixed division should validate");
}

// ===== COMPARISON OPERATIONS =====

#[test]
fn test_integer_comparison() {
    let code = "10 > 5";
    compile_and_validate(code).expect("Integer comparison should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_comparison() {
    let code = "3.14 > 2.71";
    compile_and_validate(code).expect("Float comparison should validate");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_mixed_comparison() {
    let code = "3.14 > 3";
    compile_and_validate(code).expect("Mixed comparison should validate");
}

// ===== TYPE INFERENCE SCENARIOS =====

#[test]
#[ignore] // Enable when type inference implemented
fn test_type_promotion_in_let() {
    let code = r#"
        let x = 10
        let y = 3.14
        let result = x + y
        result
    "#;
    compile_and_validate(code).expect("Type promotion in let binding should work");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_chained_operations() {
    let code = "1 + 2.0 + 3 + 4.0";
    compile_and_validate(code).expect("Chained mixed operations should validate");
}

// ===== REGRESSION TESTS =====

#[test]
fn test_multi_expression_integer_block() {
    // Ensure stack management still works after type inference changes
    let code = r#"
        2 + 2
        10 * 5
        100 - 25
    "#;
    compile_and_validate(code).expect("Multi-expression integer block should still work");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_multi_expression_float_block() {
    let code = r#"
        2.5 + 2.5
        10.0 * 5.0
        100.0 - 25.0
    "#;
    compile_and_validate(code).expect("Multi-expression float block should work");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_multi_expression_mixed_block() {
    let code = r#"
        2 + 2
        10.0 * 5.0
        100 - 25.5
    "#;
    compile_and_validate(code).expect("Multi-expression mixed block should work");
}

// ===== EDGE CASES =====

#[test]
#[ignore] // Enable when type inference implemented
fn test_zero_float() {
    let code = "0.0 + 1.0";
    compile_and_validate(code).expect("Zero float should work");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_negative_float() {
    let code = "-3.14 + 1.0";
    compile_and_validate(code).expect("Negative float should work");
}

#[test]
#[ignore] // Enable when type inference implemented
fn test_float_in_if_condition() {
    let code = r#"
        if 3.14 > 2.71 {
            1
        } else {
            0
        }
    "#;
    compile_and_validate(code).expect("Float comparison in if condition should work");
}
