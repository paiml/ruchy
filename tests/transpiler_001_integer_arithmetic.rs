//! TRANSPILER-001: Integer arithmetic broken (a + b → format!())
//!
//! BUG: Transpiler transforms numeric addition `a + b` into string concatenation `format!("{}{}", a, b)`
//! ERROR: Type mismatch - expected i32, found String
//!
//! ROOT CAUSE: Binary operator transpilation doesn't distinguish between numeric and string operations
//!
//! IMPACT: Cannot perform basic arithmetic in Ruchy code
//!
//! EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: Basic integer addition
#[test]
fn test_transpiler_001_01_integer_addition() {
    let code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Should transpile integer addition, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // CRITICAL: Must NOT contain format! or string concatenation
    assert!(
        !rust_code.contains("format!"),
        "BUG: Integer addition transpiled to format!() string concat:\n{}",
        rust_code
    );

    // Must use actual + operator
    assert!(
        rust_code.contains("a + b") || rust_code.contains("a+b"),
        "Should preserve numeric + operator, got:\n{}",
        rust_code
    );

    // Verify rustc compilation succeeds
    std::fs::write("/tmp/transpiler_001_01_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_001_01_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Integer addition fails rustc compilation:\n{}\n\nGenerated code:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 2: Integer subtraction
#[test]
fn test_transpiler_001_02_integer_subtraction() {
    let code = r#"
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Subtraction should not use format!:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("a - b") || rust_code.contains("a-b"),
        "Should preserve - operator, got:\n{}",
        rust_code
    );
}

/// Test 3: Integer multiplication
#[test]
fn test_transpiler_001_03_integer_multiplication() {
    let code = r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Multiplication should not use format!:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("a * b") || rust_code.contains("a*b"),
        "Should preserve * operator, got:\n{}",
        rust_code
    );
}

/// Test 4: Integer division
#[test]
fn test_transpiler_001_04_integer_division() {
    let code = r#"
pub fn divide(a: i32, b: i32) -> i32 {
    a / b
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Division should not use format!:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("a / b") || rust_code.contains("a/b"),
        "Should preserve / operator, got:\n{}",
        rust_code
    );
}

/// Test 5: Complex arithmetic expression
#[test]
fn test_transpiler_001_05_complex_arithmetic() {
    let code = r#"
pub fn calculate(a: i32, b: i32, c: i32) -> i32 {
    (a + b) * c - 10
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Complex arithmetic should not use format!:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_001_05_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_001_05_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Complex arithmetic fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 6: Arithmetic in struct method
#[test]
fn test_transpiler_001_06_arithmetic_in_method() {
    let code = r#"
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter { value }
    }

    pub fn add(&self, amount: i32) -> i32 {
        self.value + amount
    }

    pub fn double(&self) -> i32 {
        self.value * 2
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Method arithmetic should not use format!:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_001_06_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_001_06_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Method arithmetic fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 7: String concatenation - FIXED (TRANSPILER-004)
/// Verifies that String + String uses format!() or proper borrowing
#[test]
fn test_transpiler_001_07_string_concat_uses_format() {
    let code = r#"
pub fn concat(a: String, b: String) -> String {
    a + b
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // String concatenation CAN use format! or String::from() + &str
    // This test just verifies it compiles correctly
    std::fs::write("/tmp/transpiler_001_07_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_001_07_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "String concatenation should compile:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 8: Assignment with arithmetic - FIXED (TRANSPILER-005)
/// Verifies that mut keyword is preserved in parameter transpilation
#[test]
fn test_transpiler_001_08_assignment_with_arithmetic() {
    let code = r#"
pub fn increment_by(mut value: i32, amount: i32) -> i32 {
    value = value + amount;
    value
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("format!"),
        "BUG: Assignment arithmetic should not use format!:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_001_08_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_001_08_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Assignment arithmetic fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}
