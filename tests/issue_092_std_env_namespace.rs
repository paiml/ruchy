// Issue #92: std::env namespace for CLI argument access
//
// Problem: User wants to access command-line arguments via std::env::args()
//          but gets error "Object has no field named 'env'"
//
// Root Cause: std namespace exists (std::time, std::process, std::fs)
//             but missing std::env module
//
// Solution: Add env module to std namespace in builtin_init.rs
//           Register args() function pointing to __builtin_env_args__
//
// Test Strategy: E2E tests verifying std::env::args() works from Ruchy code
//                Compare with flat builtin env_args() for backward compatibility

use assert_cmd::Command;
use predicates::prelude::*;

/// Test 1: `std::env::args()` returns process arguments (binary, script path, etc.)
///
/// This test verifies the primary use case from Issue #92:
/// - User can call `std::env::args()` to get command-line arguments
/// - Result matches flat builtin `env_args()` for backward compatibility
/// - Returns array of strings with at least binary path and script path
#[test]
fn test_issue_092_std_env_args_basic() {
    let script_code = r#"
fun main() {
    // Get args using std::env namespace (Issue #92 request)
    let args = std::env::args();
    println!("std::env::args() type: {:?}", type_of(args));
    println!("std::env::args() length: {}", args.len());

    // Verify it's an array
    if args.len() >= 2 {
        println!("✓ std::env::args() returned array with expected length");
    } else {
        println!("✗ std::env::args() returned too few arguments");
    }

    // Compare with flat builtin for backward compatibility
    let args_flat = env_args();
    if args == args_flat {
        println!("✓ std::env::args() matches env_args() (backward compatible)");
    } else {
        println!("✗ Mismatch between std::env::args() and env_args()");
    }
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ std::env::args() returned array with expected length"))
        .stdout(predicate::str::contains("✓ std::env::args() matches env_args() (backward compatible)"));
}

/// Test 2: `std::env::args()` array can be indexed
///
/// Verifies that the returned array is usable for common operations:
/// - Access first element (binary path)
/// - Access second element (script info or first user arg)
/// - Iterate over elements
#[test]
fn test_issue_092_std_env_args_indexing() {
    let script_code = r#"
fun main() {
    let args = std::env::args();

    // Access first element (binary path)
    if args.len() > 0 {
        let binary = args[0];
        println!("✓ Binary path accessible: {}", binary);
    }

    // Access second element
    if args.len() > 1 {
        let second = args[1];
        println!("✓ Second argument accessible: {}", second);
    }

    // Iterate over args
    println!("All arguments:");
    for i in range(0, args.len()) {
        println!("  [{}]: {}", i, args[i]);
    }
    println!("✓ Iteration works");
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ Binary path accessible"))
        .stdout(predicate::str::contains("✓ Second argument accessible"))
        .stdout(predicate::str::contains("✓ Iteration works"));
}

/// Test 3: Backward compatibility - flat builtin `env_args()` still works
///
/// Ensures that adding `std::env::args()` doesn't break existing code
/// that uses the flat builtin `env_args()`
#[test]
fn test_issue_092_backward_compatibility_flat_builtin() {
    let script_code = r#"
fun main() {
    // Old way (flat builtin) should still work
    let args = env_args();
    println!("env_args() length: {}", args.len());

    if args.len() >= 2 {
        println!("✓ Flat builtin env_args() still works");
    } else {
        println!("✗ Flat builtin env_args() broken");
    }
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ Flat builtin env_args() still works"));
}

/// Test 4: `std::env` module exists alongside other std modules
///
/// Verifies that `std::env` is properly registered and doesn't interfere
/// with existing `std::time`, `std::process`, `std::fs` modules
#[test]
fn test_issue_092_std_env_coexists_with_other_modules() {
    let script_code = r#"
fun main() {
    // Test std::env (Issue #92)
    let args = std::env::args();
    println!("✓ std::env::args() works");

    // Test std::time (STDLIB-003)
    let t = std::time::now_millis();
    println!("✓ std::time::now_millis() still works");

    // Verify env module doesn't break fs module (Issue #90)
    let exists = std::fs::exists("/tmp");
    println!("✓ std::fs::exists() still works");

    println!("All std modules coexist successfully");
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓ std::env::args() works"))
        .stdout(predicate::str::contains("✓ std::time::now_millis() still works"))
        .stdout(predicate::str::contains("✓ std::fs::exists() still works"))
        .stdout(predicate::str::contains("All std modules coexist successfully"));
}

/// Test 5: Real-world use case - script argument parsing
///
/// Simulates typical CLI tool pattern where script processes arguments
#[test]
fn test_issue_092_real_world_argument_parsing() {
    let script_code = r#"
fun main() {
    let args = std::env::args();

    // Skip first two elements (binary and script path)
    let user_args_start = 2;

    println!("Process info:");
    println!("  Binary: {}", args[0]);
    if args.len() > 1 {
        println!("  Script/mode: {}", args[1]);
    }

    // In real CLI tools, user args would start at index 2
    // For this test with -e flag, we just verify the structure
    let expected_structure = args.len() >= 2;
    if expected_structure {
        println!("✓ Argument structure correct for CLI tool pattern");
    } else {
        println!("✗ Unexpected argument structure");
    }
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Process info:"))
        .stdout(predicate::str::contains("Binary:"))
        .stdout(predicate::str::contains("✓ Argument structure correct for CLI tool pattern"));
}
