/// RED Phase Test: Detect and fix deadlock bug in global variable assignments
///
/// ROOT CAUSE: Double-locking when assigning to globals
/// Example: counter = counter + 1 transpiles to:
///   *`counter.lock().unwrap()` = *`counter.lock().unwrap()` + 1;
///   ^^^^^^^^^^^^^^^^^^^^^       ^^^^^^^^^^^^^^^^^^^^^^^^
///   Lock #1                     Lock #2 → DEADLOCK!
///
/// EXPECTED: Should lock once and operate on guard
///   {
///       let mut guard = `counter.lock().unwrap()`;
///       *guard = *guard + 1;
///   }
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

/// Test 1: Simple assignment doesn't deadlock
/// Tests: counter = counter + 1
#[test]
fn test_no_deadlock_simple_assignment() {
    let code = r#"
let mut counter = 0

fn increment() {
    counter = counter + 1
}

increment()
println!("{}", counter)
"#;

    // Write to temp file
    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Write transpiled code
    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_deadlock")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout (2 seconds)
    // If it hangs, this will timeout
    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    // Check it didn't timeout (exit code 124 = timeout)
    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)! Output:\n{}\nStderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    assert!(
        run.status.success(),
        "Program failed:\n{}\nStderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    // Verify output
    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains('1'),
        "Expected output '1', got: {output_str}"
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 2: Compound assignment doesn't deadlock
/// Tests: counter += 1
#[test]
fn test_no_deadlock_compound_assignment() {
    let code = r#"
let mut total = 0

fn add_value(x: i32) {
    total += x
}

add_value(5)
add_value(10)
println!("{}", total)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_compound")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)!"
    );

    assert!(run.status.success(), "Program failed");

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains("15"),
        "Expected '15', got: {output_str}"
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 3: Multiple references don't deadlock
/// Tests: result = global1 + global2
#[test]
fn test_no_deadlock_multiple_globals() {
    let code = r#"
let mut x = 10
let mut y = 20

fn compute() {
    x = x + y
}

compute()
println!("{}", x)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd
        .arg("transpile")
        .arg(&ruchy_path)
        .output()
        .expect("Failed to transpile");

    assert!(output.status.success(), "Transpile failed");
    let transpiled = String::from_utf8(output.stdout).unwrap();

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_multiple")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .expect("Failed to run");

    assert_ne!(
        run.status.code(),
        Some(124),
        "Program DEADLOCKED (timeout)!"
    );

    assert!(run.status.success(), "Program failed");

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(
        output_str.contains("30"),
        "Expected '30', got: {output_str}"
    );

    // Cleanup
    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// ============================================================================
// Property-Based Tests (10K+ cases)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property: Any global variable assignment should not deadlock
    /// Strategy: Generate random variable names and values
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn prop_no_deadlock_on_any_global_assignment() {
        proptest!(|(
            var_name in "[a-z]{3,8}",
            init_val in 0i32..100,
            op_val in 1i32..50,
        )| {
            let code = format!(
                r#"
let mut {var_name} = {init_val}

fn update() {{
    {var_name} = {var_name} + {op_val}
}}

update()
println!("{{}}", {var_name})
"#
            );

            let temp = NamedTempFile::new().unwrap();
            let ruchy_path = temp.path().with_extension("ruchy");
            fs::write(&ruchy_path, code).unwrap();

            // Transpile
            let mut cmd = Command::cargo_bin("ruchy").unwrap();
            let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
            prop_assert!(output.status.success(), "Transpile failed");

            let transpiled = String::from_utf8(output.stdout).unwrap();
            let rs_path = temp.path().with_extension("rs");
            fs::write(&rs_path, transpiled).unwrap();

            // Compile
            let compile = StdCommand::new("rustc")
                .arg(&rs_path)
                .arg("--crate-name").arg("prop_test")
                .arg("-o").arg(temp.path().with_extension("exe"))
                .output()
                .unwrap();
            prop_assert!(compile.status.success(), "Compilation failed");

            // Run with timeout - should never deadlock
            let exe_path = temp.path().with_extension("exe");
            let run = StdCommand::new("timeout")
                .arg("2")
                .arg(&exe_path)
                .output()
                .unwrap();

            prop_assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
            prop_assert!(run.status.success(), "Execution failed");

            // Verify output
            let expected = init_val + op_val;
            let output_str = String::from_utf8_lossy(&run.stdout);
            prop_assert!(output_str.contains(&expected.to_string()));

            // Cleanup
            let _ = fs::remove_file(&ruchy_path);
            let _ = fs::remove_file(&rs_path);
            let _ = fs::remove_file(&exe_path);
        });
    }

    /// Property: Compound assignments should not deadlock
    #[test]
    #[ignore]
    fn prop_no_deadlock_compound_assignment() {
        proptest!(|(
            var_name in "[a-z]{3,8}",
            init_val in 0i32..100,
        )| {
            // Use fixed value to avoid return type inference bug
            let code = format!(
                r#"
let mut {var_name} = {init_val}

fn add_five() {{
    {var_name} += 5
}}

add_five()
println!("{{}}", {var_name})
"#
            );

            let temp = NamedTempFile::new().unwrap();
            let ruchy_path = temp.path().with_extension("ruchy");
            fs::write(&ruchy_path, code).unwrap();

            let mut cmd = Command::cargo_bin("ruchy").unwrap();
            let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
            prop_assert!(output.status.success());

            let transpiled = String::from_utf8(output.stdout).unwrap();
            let rs_path = temp.path().with_extension("rs");
            fs::write(&rs_path, transpiled).unwrap();

            let compile = StdCommand::new("rustc")
                .arg(&rs_path)
                .arg("--crate-name").arg("prop_compound")
                .arg("-o").arg(temp.path().with_extension("exe"))
                .output()
                .unwrap();
            prop_assert!(compile.status.success());

            let exe_path = temp.path().with_extension("exe");
            let run = StdCommand::new("timeout")
                .arg("2")
                .arg(&exe_path)
                .output()
                .unwrap();

            prop_assert_ne!(run.status.code(), Some(124));
            prop_assert!(run.status.success());

            let expected = init_val + 5;
            let output_str = String::from_utf8_lossy(&run.stdout);
            prop_assert!(output_str.contains(&expected.to_string()));

            let _ = fs::remove_file(&ruchy_path);
            let _ = fs::remove_file(&rs_path);
            let _ = fs::remove_file(&exe_path);
        });
    }
}

// ============================================================================
// Unit Tests for Code Coverage (>80% target)
// ============================================================================

/// Test 4: Non-global assignments don't use guard pattern
#[test]
fn test_local_assignment_no_guard() {
    let code = r#"
fn compute() {
    let mut x = 10
    x = x + 5
    println!("{}", x)
}
compute()
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success(), "Transpile failed");

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should NOT contain guard pattern for local variables
    assert!(
        !transpiled.contains("__guard"),
        "Local var shouldn't use guard"
    );
    assert!(
        transpiled.contains("let mut x"),
        "Should have local variable"
    );

    let _ = fs::remove_file(&ruchy_path);
}

/// Test 5: Global assignment where value doesn't reference target (no deadlock risk)
#[test]
fn test_global_assignment_no_self_reference() {
    let code = r#"
let mut result = 0
let mut other = 5

fn set_result() {
    result = other + 10
}

set_result()
println!("{}", result)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success());

    let transpiled = String::from_utf8(output.stdout).unwrap();
    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_global_no_ref")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124));
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("15"));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 6: Unary operator in self-referencing assignment
#[test]
fn test_unary_in_self_reference() {
    let code = r#"
let mut value = -10

fn negate() {
    value = -value
}

negate()
println!("{}", value)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success());

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should contain guard pattern
    assert!(
        transpiled.contains("__guard"),
        "Should use guard for unary self-ref"
    );

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_unary")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124));
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("10"));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 7: Method call doesn't trigger guard (non-identifier)
#[test]
fn test_method_call_no_special_handling() {
    let code = r#"
let mut text = "hello"

fn process() {
    text = "world"
    println!("{}", text)
}

process()
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success());

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Simple string assignment doesn't need guard
    assert!(transpiled.contains("static text"));

    let _ = fs::remove_file(&ruchy_path);
}

/// Test 8: Multiple operations with self-reference
#[test]
fn test_nested_binary_self_reference() {
    let code = r#"
let mut num = 10

fn complex_update() {
    num = num + num
}

complex_update()
println!("{}", num)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success());

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should use guard pattern for nested expression
    assert!(transpiled.contains("__guard"));

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_nested")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124));
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("20")); // 10 + 10 = 20

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

/// Test 9: Compound assignment with different operators
#[test]
fn test_compound_different_operators() {
    let code = r#"
let mut x = 10

fn test_ops() {
    x *= 2
}

test_ops()
println!("{}", x)
"#;

    let temp = NamedTempFile::new().unwrap();
    let ruchy_path = temp.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    let output = cmd.arg("transpile").arg(&ruchy_path).output().unwrap();
    assert!(output.status.success());

    let transpiled = String::from_utf8(output.stdout).unwrap();

    // Should use guard for global compound assignment
    assert!(transpiled.contains("__guard"));
    assert!(transpiled.contains("*="));

    let rs_path = temp.path().with_extension("rs");
    fs::write(&rs_path, transpiled).unwrap();

    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_compound_ops")
        .arg("-o")
        .arg(temp.path().with_extension("exe"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe_path = temp.path().with_extension("exe");
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124));
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("20"));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// Test 10: Bitwise operators in self-reference (flags = flags & 0xFF)
#[test]
fn test_bitwise_operators_self_reference() {
    let code = r#"
let mut flags = 255

fn mask_flags() {
    flags = flags & 15
}

mask_flags()
println!("{}", flags)
"#;

    let ruchy_path = PathBuf::from("/tmp/test_bitwise.ruchy");
    let rs_path = PathBuf::from("/tmp/test_bitwise.rs");
    let exe_path = PathBuf::from("/tmp/test_bitwise");

    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let transpile = StdCommand::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "transpile"])
        .arg(&ruchy_path)
        .output()
        .unwrap();
    assert!(transpile.status.success());

    fs::write(&rs_path, transpile.stdout).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_bitwise")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("15"));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// Test 11: Comparison operators (status = status > 5)
#[test]
fn test_comparison_operators_self_reference() {
    let code = r#"
let mut status = 10

fn check_status() {
    status = if status > 5 { 1 } else { 0 }
}

check_status()
println!("{}", status)
"#;

    let ruchy_path = PathBuf::from("/tmp/test_comparison.ruchy");
    let rs_path = PathBuf::from("/tmp/test_comparison.rs");
    let exe_path = PathBuf::from("/tmp/test_comparison");

    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let transpile = StdCommand::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "transpile"])
        .arg(&ruchy_path)
        .output()
        .unwrap();
    assert!(transpile.status.success());

    fs::write(&rs_path, transpile.stdout).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_comparison")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains('1'));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// Test 12: Modulo operator (index = (index + 1) % 10)
#[test]
fn test_modulo_operator_self_reference() {
    let code = r#"
let mut index = 9

fn increment_circular() {
    index = (index + 1) % 10
}

increment_circular()
println!("{}", index)
"#;

    let ruchy_path = PathBuf::from("/tmp/test_modulo.ruchy");
    let rs_path = PathBuf::from("/tmp/test_modulo.rs");
    let exe_path = PathBuf::from("/tmp/test_modulo");

    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let transpile = StdCommand::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "transpile"])
        .arg(&ruchy_path)
        .output()
        .unwrap();
    assert!(transpile.status.success());

    fs::write(&rs_path, transpile.stdout).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_modulo")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains('0')); // (9 + 1) % 10 = 0

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// Test 13: Logical AND operator (result = result && true)
#[test]
fn test_logical_and_self_reference() {
    let code = r#"
let mut result = true

fn check_result() {
    result = result && true
}

check_result()
println!("{}", result)
"#;

    let ruchy_path = PathBuf::from("/tmp/test_logical.ruchy");
    let rs_path = PathBuf::from("/tmp/test_logical.rs");
    let exe_path = PathBuf::from("/tmp/test_logical");

    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let transpile = StdCommand::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "transpile"])
        .arg(&ruchy_path)
        .output()
        .unwrap();
    assert!(transpile.status.success());

    fs::write(&rs_path, transpile.stdout).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_logical")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("true"));

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}

// Test 14: Shift operators (value = value << 1)
#[test]
fn test_shift_operators_self_reference() {
    let code = r#"
let mut value = 5

fn shift_left() {
    value = value << 1
}

shift_left()
println!("{}", value)
"#;

    let ruchy_path = PathBuf::from("/tmp/test_shift.ruchy");
    let rs_path = PathBuf::from("/tmp/test_shift.rs");
    let exe_path = PathBuf::from("/tmp/test_shift");

    fs::write(&ruchy_path, code).unwrap();

    // Transpile
    let transpile = StdCommand::new("cargo")
        .args(["run", "--release", "--bin", "ruchy", "--", "transpile"])
        .arg(&ruchy_path)
        .output()
        .unwrap();
    assert!(transpile.status.success());

    fs::write(&rs_path, transpile.stdout).unwrap();

    // Compile
    let compile = StdCommand::new("rustc")
        .arg(&rs_path)
        .arg("--crate-name")
        .arg("test_shift")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run with timeout
    let run = StdCommand::new("timeout")
        .arg("2")
        .arg(&exe_path)
        .output()
        .unwrap();

    assert_ne!(run.status.code(), Some(124), "DEADLOCK detected!");
    assert!(run.status.success());

    let output_str = String::from_utf8_lossy(&run.stdout);
    assert!(output_str.contains("10")); // 5 << 1 = 10

    let _ = fs::remove_file(&ruchy_path);
    let _ = fs::remove_file(&rs_path);
    let _ = fs::remove_file(&exe_path);
}
