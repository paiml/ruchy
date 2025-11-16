//! TRANSPILER-TYPE-INFER-PARAMS: Infer return type from parameter types
//!
//! **Bug**: Functions returning parameter values default to i32 instead of parameter type
//! **Found By**: Property test suite (`property_01_type_inference_correctness`)
//! **Example**: fun a(a: f64) { a } transpiles to fn a(a: f64) -> i32 (WRONG!)
//! **Expected**: Should infer -> f64 from parameter type

use assert_cmd::Command;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Function returning f64 parameter should infer f64 return type
#[test]
fn test_transpiler_type_infer_001_f64_param_return() {
    let code = r"
fun a(a: f64) {
    let result = a;
    result
}
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Transpile failed");
    let rust_str = rust_code.to_string();

    // Should infer -> f64, NOT -> i32
    assert!(
        rust_str.contains("-> f64"),
        "Expected '-> f64' but got: {rust_str}"
    );
    assert!(
        !rust_str.contains("-> i32"),
        "Should NOT default to i32 for f64 param: {rust_str}"
    );
}

/// Test 2: Function directly returning string parameter
#[test]
fn test_transpiler_type_infer_002_str_param_return() {
    let code = r"
fun echo(msg: str) {
    msg
}
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Transpile failed");
    let rust_str = rust_code.to_string();

    // Should infer -> &str
    assert!(
        rust_str.contains("-> &str") || rust_str.contains("-> & str"),
        "Expected '-> &str' but got: {rust_str}"
    );
}

/// Test 3: Function returning bool parameter
#[test]
fn test_transpiler_type_infer_003_bool_param_return() {
    let code = r"
fun identity(flag: bool) {
    flag
}
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Transpile failed");
    let rust_str = rust_code.to_string();

    // Should infer -> bool
    assert!(
        rust_str.contains("-> bool"),
        "Expected '-> bool' but got: {rust_str}"
    );
}

/// Test 4: Function returning i32 parameter (should still work)
#[test]
fn test_transpiler_type_infer_004_i32_param_return() {
    let code = r"
fun pass_through(x: i32) {
    x
}
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Transpile failed");
    let rust_str = rust_code.to_string();

    // Should infer -> i32 (correct)
    assert!(
        rust_str.contains("-> i32"),
        "Expected '-> i32' but got: {rust_str}"
    );
}

/// Test 5: Full compilation test - f64 function must compile and execute
/// TRANSPILER-TYPE-INFER-EXPR: Currently FAILING - needs full expression type inference
/// Ticket: See docs/execution/roadmap.yaml TRANSPILER-TYPE-INFER-EXPR
#[test]
fn test_transpiler_type_infer_005_f64_compile_execute() {
    let code = r#"
fun double_value(x: f64) {
    let result = x * 2.0;
    result
}

fun main() {
    println!("{}", double_value(3.14))
}
"#;

    let temp = tempfile::TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");
    let binary = temp.path().join("test_binary");

    std::fs::write(&source, code).expect("Failed to write file");

    // Must compile successfully
    let result = ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("Failed to run ruchy");

    assert!(
        result.status.success(),
        "Compilation should succeed:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Execute and verify
    let exec_result = Command::new(&binary)
        .output()
        .expect("Failed to execute binary");

    assert!(exec_result.status.success());
    let output = String::from_utf8_lossy(&exec_result.stdout);
    assert!(output.contains("6.28") || output.contains("6.3"));
}
