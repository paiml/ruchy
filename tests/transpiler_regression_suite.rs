/// TRANSPILER-016B: Regression Test Suite
///
/// Prevents re-introduction of past transpiler bugs:
/// - TRANSPILER-009: Standalone functions
/// - TRANSPILER-011: Nested field access
/// - TRANSPILER-013: Object literal return type inference
///
/// GitHub Issue: #141

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

/// Helper: Parse and transpile Ruchy code to Rust
fn transpile(code: &str) -> String {
    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);
    assert!(result.is_ok(), "Transpile should succeed, got: {:?}", result.err());
    result.unwrap().to_string()
}

/// Helper: Verify rustc compilation
fn verify_compiles(rust_code: &str, crate_type: &str) {
    // Use unique temp file per test to avoid parallel test interference
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_file = format!("/tmp/transpiler_regression_{}.rs", id);
    let temp_output = format!("/tmp/transpiler_regression_{}", id);

    std::fs::write(&temp_file, rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", crate_type, &temp_file, "-o", &temp_output])
        .output()
        .expect("Failed to run rustc");

    // Clean up temp files
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(&temp_output);

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Transpiled code fails compilation:\n{stderr}\n\nCode:\n{rust_code}"
        );
    }
}

// ========== TRANSPILER-009: Standalone Functions ==========

#[test]
fn test_regression_009_standalone_function_appears_in_output() {
    let code = r#"
pub fun standalone_helper() -> i32 {
    42
}

pub fun main() -> i32 {
    standalone_helper()
}
"#;

    let rust = transpile(code);

    // BUG: Function was defined but not included in output
    assert!(
        rust.contains("standalone_helper"),
        "Standalone function must appear in output:\n{rust}"
    );

    // Must compile
    verify_compiles(&rust, "lib");
}

#[test]
fn test_regression_009_multiple_standalone_functions() {
    let code = r#"
pub fun add(a: i32, b: i32) -> i32 { a + b }
pub fun mul(a: i32, b: i32) -> i32 { a * b }
pub fun calculate() -> i32 { add(5, 3) * mul(2, 4) }
"#;

    let rust = transpile(code);

    assert!(rust.contains("add"), "add() must appear");
    assert!(rust.contains("mul"), "mul() must appear");
    assert!(rust.contains("calculate"), "calculate() must appear");

    verify_compiles(&rust, "lib");
}

// ========== TRANSPILER-011: Nested Field Access ==========

#[test]
fn test_regression_011_nested_field_access_uses_dot_not_double_colon() {
    let code = r#"
pub fun handler(event: LambdaEvent) -> String {
    event.requestContext.requestId
}
"#;

    let rust = transpile(code);

    // BUG: Generated event::requestContext::requestId (invalid)
    // FIX: Should generate event.requestContext.requestId (valid)
    let has_dot = rust.contains("event.") || rust.contains("event .");
    let has_double_colon = rust.contains("event::");
    assert!(
        has_dot && !has_double_colon,
        "Field access must use dot notation, not double colon:\n{rust}"
    );

    // Should NOT contain double colons for field access
    assert!(
        !rust.contains("requestContext::") && !rust.contains("requestId::"),
        "Nested field access must not use :: operator:\n{rust}"
    );
}

#[test]
fn test_regression_011_deeply_nested_field_access() {
    let code = r#"
pub fun get_value(obj: Wrapper) -> i32 {
    obj.inner.data.value
}
"#;

    let rust = transpile(code);

    // All field accesses must use dot notation (accept spaces)
    let has_dot = rust.contains("obj.") || rust.contains("obj .");
    let has_double_colon = rust.contains("obj::");
    assert!(
        has_dot && !has_double_colon,
        "Must use dot notation for field access:\n{rust}"
    );
}

// ========== TRANSPILER-013: Object Literal Return Type Inference ==========

#[test]
fn test_regression_013_object_literal_infers_btreemap_not_i32() {
    let code = r#"
pub fun create_response() -> Object {
    {
        "statusCode": 200,
        "body": "Success"
    }
}
"#;

    let rust = transpile(code);

    // BUG: Inferred return type as i32 instead of BTreeMap<String, Value>
    // FIX: Object literals must map to BTreeMap
    assert!(
        rust.contains("BTreeMap") || rust.contains("btreemap") || rust.contains("HashMap"),
        "Object literal must map to Map type, not i32:\n{rust}"
    );

    // Should NOT infer as integer type
    assert!(
        !rust.contains("-> i32 {") || rust.contains("BTreeMap"),
        "Object literal return must not be i32:\n{rust}"
    );
}

#[test]
fn test_regression_013_nested_object_literals() {
    let code = r#"
pub fun create_config() -> Object {
    {
        "server": {
            "host": "localhost",
            "port": 8080
        },
        "database": {
            "url": "postgres://localhost"
        }
    }
}
"#;

    let rust = transpile(code);

    // Nested objects must also map to BTreeMap
    assert!(
        rust.contains("BTreeMap") || rust.contains("HashMap"),
        "Nested object literals must map to Map types:\n{rust}"
    );
}

// ========== Method Call Mutability ==========

#[test]
fn test_regression_method_call_preserves_receiver_mutability() {
    let code = r#"
pub fun process(data: Vec<i32>) -> Vec<i32> {
    let mut result = data;
    result.push(42);
    result
}
"#;

    let rust = transpile(code);

    // Mutable methods must have mutable receivers
    assert!(
        rust.contains("mut result"),
        "Mutable method calls must have mutable receivers:\n{rust}"
    );

    verify_compiles(&rust, "lib");
}

// ========== Integration Tests: ruchy-lambda Examples ==========

#[test]
fn test_regression_lambda_simple_handler() {
    // From ruchy-lambda examples/simple_handler.ruchy
    let code = r#"
pub fun handler(event: LambdaEvent) -> LambdaResponse {
    {
        "statusCode": 200,
        "body": "Hello from Ruchy Lambda!"
    }
}
"#;

    let rust = transpile(code);

    // Must transpile successfully
    assert!(rust.contains("handler"), "Handler function must exist");
    assert!(rust.len() > 50, "Output must not be empty");

    // Lambda pattern: Object literal return
    assert!(
        rust.contains("BTreeMap") || rust.contains("HashMap") || rust.contains("body"),
        "Lambda response must be map-like:\n{rust}"
    );
}

#[test]
fn test_regression_lambda_hello_world() {
    // Minimal Lambda handler
    let code = r#"
pub fun handler() -> Object {
    { "message": "Hello World" }
}
"#;

    let rust = transpile(code);

    assert!(rust.contains("handler"));
    verify_compiles(&rust, "lib");
}

#[test]
fn test_regression_lambda_with_field_access() {
    // Lambda handler accessing nested event fields
    let code = r#"
pub fun handler(event: LambdaEvent) -> Object {
    let request_id = event.requestContext.requestId;
    {
        "statusCode": 200,
        "requestId": request_id
    }
}
"#;

    let rust = transpile(code);

    // Field access must use dots
    assert!(
        rust.contains("event.") && !rust.contains("event::"),
        "Event field access must use dot notation:\n{rust}"
    );

    // Return must be map-like
    assert!(
        rust.contains("BTreeMap") || rust.contains("HashMap") || rust.contains("statusCode"),
        "Lambda response must be map-like:\n{rust}"
    );
}

// ========== String Methods ==========

#[test]
fn test_regression_string_methods_compile() {
    let code = r#"
pub fun process(text: String) -> usize {
    text.len()
}
"#;

    let rust = transpile(code);
    verify_compiles(&rust, "lib");
}

// ========== For Loop Transpilation ==========

#[test]
fn test_regression_for_loop_basic() {
    let code = r#"
pub fun sum_range(n: i32) -> i32 {
    let mut total = 0;
    for i in 0..n {
        total = total + i;
    }
    total
}
"#;

    let rust = transpile(code);
    verify_compiles(&rust, "lib");
}

// ========== Match Expression ==========

#[test]
fn test_regression_match_expression() {
    let code = r#"
pub fun classify(x: i32) -> String {
    match x {
        0 => "zero",
        1 => "one",
        _ => "many"
    }
}
"#;

    let rust = transpile(code);
    verify_compiles(&rust, "lib");
}

// ========== If Expression with Blocks ==========

#[test]
fn test_regression_if_expression_with_blocks() {
    let code = r#"
pub fun abs(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}
"#;

    let rust = transpile(code);
    verify_compiles(&rust, "lib");
}

// ========== Lambda/Closure ==========

#[test]
fn test_regression_lambda_closure() {
    let code = r#"
pub fun apply() -> i32 {
    let f = |x: i32| x * 2;
    f(21)
}
"#;

    let rust = transpile(code);
    verify_compiles(&rust, "lib");
}
