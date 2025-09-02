// Integration test for enum variant values feature (GitHub Issue #18)
// Verifies complete end-to-end functionality for TypeScript migration support

use std::process::Command;
use std::fs;

#[test]
fn test_enum_with_values_full_pipeline() {
    // Create a test Ruchy file with enum that has values
    let test_file = "/tmp/test_enum_values.ruchy";
    let test_content = r#"
pub enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
    FATAL = 4
}

pub enum HttpStatus {
    OK = 200,
    BadRequest = 400,
    NotFound = 404,
    ServerError = 500
}

fn main() {
    // Note: Enum variant construction is a separate feature (RUCHY-203)
    // For now, just test that the enum definition works
    println("Enum with values defined successfully")
}
"#;

    fs::write(test_file, test_content).expect("Failed to write test file");

    // Test 1: Check syntax validation
    let check_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "check", test_file])
        .output()
        .expect("Failed to run check command");

    assert!(check_output.status.success(), 
            "Syntax check failed: {}", String::from_utf8_lossy(&check_output.stderr));

    // Test 2: Transpile to Rust
    let transpile_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "transpile", test_file])
        .output()
        .expect("Failed to run transpile command");

    let rust_code = String::from_utf8_lossy(&transpile_output.stdout);
    
    // Verify transpiled code has correct attributes and values
    assert!(rust_code.contains("#[repr") || rust_code.contains("# [repr"), 
            "Missing repr attribute in transpiled code");
    assert!(rust_code.contains("DEBUG = 0"), 
            "Missing DEBUG = 0 in transpiled code");
    assert!(rust_code.contains("INFO = 1"), 
            "Missing INFO = 1 in transpiled code");
    assert!(rust_code.contains("OK = 200"), 
            "Missing OK = 200 in transpiled code");
}

#[test]
fn test_mixed_enum_types() {
    // Test that regular enums and valued enums can coexist
    let test_file = "/tmp/test_mixed_enums.ruchy";
    let test_content = r#"
// Regular enum without values
enum Color {
    Red,
    Green,
    Blue
}

// Enum with explicit values
enum Priority {
    Low = 1,
    Medium = 5,
    High = 10
}

// Generic enum (should work without values)
enum Option<T> {
    Some(T),
    None
}
"#;

    fs::write(test_file, test_content).expect("Failed to write test file");

    let check_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "check", test_file])
        .output()
        .expect("Failed to run check command");

    assert!(check_output.status.success(), 
            "Mixed enum types check failed: {}", String::from_utf8_lossy(&check_output.stderr));
}

#[test]
fn test_typescript_migration_scenario() {
    // Real-world TypeScript migration scenario from ubuntu-config-scripts
    let test_file = "/tmp/test_typescript_migration.ruchy";
    let test_content = r#"
pub enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3
}

pub struct LoggerOptions {
    level: LogLevel,
    prefix: String,
    use_colors: bool
}

impl LoggerOptions {
    pub fn new() -> LoggerOptions {
        LoggerOptions {
            level: LogLevel::INFO,
            prefix: "[LOG]".to_string(),
            use_colors: true
        }
    }
    
    pub fn with_level(mut self, level: LogLevel) -> LoggerOptions {
        self.level = level
        self
    }
}
"#;

    fs::write(test_file, test_content).expect("Failed to write test file");

    // Should parse and check successfully
    let check_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "check", test_file])
        .output()
        .expect("Failed to run check command");

    assert!(check_output.status.success(), 
            "TypeScript migration scenario failed: {}", String::from_utf8_lossy(&check_output.stderr));

    // Transpile and verify
    let transpile_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "transpile", test_file])
        .output()
        .expect("Failed to run transpile command");

    let rust_code = String::from_utf8_lossy(&transpile_output.stdout);
    
    // Verify enum with values is transpiled correctly
    assert!(rust_code.contains("LogLevel"), "Missing LogLevel enum");
    assert!(rust_code.contains("DEBUG = 0"), "Missing discriminant values");
    assert!(rust_code.contains("repr"), "Missing repr attribute");
}

#[test]
fn test_negative_discriminant_values() {
    // Test that negative values work
    let test_file = "/tmp/test_negative_values.ruchy";
    let test_content = r#"
enum Temperature {
    Freezing = -273,
    Cold = -10,
    Normal = 20,
    Hot = 40
}
"#;

    fs::write(test_file, test_content).expect("Failed to write test file");

    let check_output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "check", test_file])
        .output()
        .expect("Failed to run check command");

    assert!(check_output.status.success(), 
            "Negative values check failed: {}", String::from_utf8_lossy(&check_output.stderr));
}

#[test]
fn test_enum_values_backwards_compatibility() {
    // Ensure old enum syntax still works
    let test_cases = vec![
        "enum Simple { A, B, C }",
        "pub enum Public { X, Y }",
        "enum Generic<T> { Some(T), None }",
        "enum Result<T, E> { Ok(T), Err(E) }",
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        let test_file = format!("/tmp/test_compat_{}.ruchy", i);
        fs::write(&test_file, test_case).expect("Failed to write test file");

        let check_output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", "check", &test_file])
            .output()
            .expect("Failed to run check command");

        assert!(check_output.status.success(), 
                "Backwards compatibility failed for: {}", test_case);
    }
}

#[test] 
fn test_enum_values_error_cases() {
    // Test that invalid enum value syntax is rejected
    let invalid_cases = vec![
        ("enum Bad { A = \"string\" }", "string values not allowed"),
        ("enum Bad { A = 1.5 }", "float values not allowed"),
        ("enum Bad { A(i32) = 1 }", "can't have both fields and values"),
    ];

    for (i, (test_case, description)) in invalid_cases.iter().enumerate() {
        let test_file = format!("/tmp/test_invalid_{}.ruchy", i);
        fs::write(&test_file, test_case).expect("Failed to write test file");

        let check_output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", "check", &test_file])
            .output()
            .expect("Failed to run check command");

        assert!(!check_output.status.success(), 
                "Should have rejected {}: {}", description, test_case);
    }
}