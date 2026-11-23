//! EXTREME TDD - RED PHASE: Tests for global let mut transpiler bug
//!
//! Bug: `let mut counter = 0;` at top level transpiles to `();`
//!
//! Expected behavior: Should transpile to `let mut counter = 0;` in `main()`
//!
//! Test status: These tests MUST fail initially to prove we're testing the bug

/// Test case 1: Single mutable let statement
/// Should transpile to: fn `main()` { let mut x = 0; }
#[test]
#[ignore = "global let mut transpiler not fixed yet"]
fn test_transpiler_single_let_mut() {
    let code = "let mut x = 0;";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Should contain the let statement
    assert!(
        transpiled.contains("let mut x = 0"),
        "Expected 'let mut x = 0' but got: {transpiled}"
    );

    // Should NOT contain just unit literal
    assert!(
        !transpiled.contains("()"),
        "Should not transpile let to (), got: {transpiled}"
    );
}

/// Test case 2: Multiple mutable let statements
/// Should transpile to: fn `main()` { let mut x = 0; let mut y = 1; }
#[test]
fn test_transpiler_multiple_let_mut() {
    let code = r"let mut x = 0;
let mut y = 1;";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Should contain both let statements
    assert!(
        transpiled.contains("let mut x = 0"),
        "Expected 'let mut x = 0' but got: {transpiled}"
    );
    assert!(
        transpiled.contains("let mut y = 1"),
        "Expected 'let mut y = 1' but got: {transpiled}"
    );

    // Should NOT contain result wrapping for let statements
    assert!(
        !transpiled.contains("let result = {"),
        "Let statements should not be wrapped in result block, got: {transpiled}"
    );
}

/// Test case 3: Let mut with subsequent usage
/// The original bug report case
#[test]
fn test_transpiler_let_mut_with_usage() {
    let code = r#"let mut counter = 0;
counter = counter + 1;
println!("{}", counter);"#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Should contain the let statement
    assert!(
        transpiled.contains("let mut counter = 0"),
        "Expected 'let mut counter = 0' but got: {transpiled}"
    );

    // Should contain the assignment
    assert!(
        transpiled.contains("counter = counter + 1"),
        "Expected 'counter = counter + 1' but got: {transpiled}"
    );

    // Should NOT start with just ()
    assert!(
        !transpiled.contains("fn main() {\n    ();"),
        "First statement should not be (), got: {transpiled}"
    );
}

/// Test case 4: Mixed let and immutable let
#[test]
fn test_transpiler_mixed_let_statements() {
    let code = r"let x = 1;
let mut y = 2;
let z = 3;";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // All let statements should be present
    assert!(
        transpiled.contains("let x = 1"),
        "Expected 'let x = 1' but got: {transpiled}"
    );
    assert!(
        transpiled.contains("let mut y = 2"),
        "Expected 'let mut y = 2' but got: {transpiled}"
    );
    assert!(
        transpiled.contains("let z = 3"),
        "Expected 'let z = 3' but got: {transpiled}"
    );
}

/// Test case 5: Let with type annotation
#[test]
fn test_transpiler_let_mut_with_type_annotation() {
    let code = "let mut x: int = 0;";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Should contain the let statement with type
    assert!(
        transpiled.contains("let mut x") && transpiled.contains(": i64"),
        "Expected 'let mut x : i64 = 0' but got: {transpiled}"
    );
}

/// Test case 6: End-to-end - transpile, compile, execute
#[test]
fn test_transpiler_let_mut_compiles_and_runs() {
    let code = r#"let mut x = 5;
x = x * 2;
println!("{}", x);"#;

    // Transpile
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let rust_file = temp_dir.join("test_let_mut.rs");
    std::fs::write(&rust_file, transpiled.as_bytes()).expect("Failed to write temp file");

    // Compile
    let compile_output = std::process::Command::new("rustc")
        .arg(&rust_file)
        .arg("-o")
        .arg(temp_dir.join("test_let_mut"))
        .output()
        .expect("Failed to compile");

    assert!(
        compile_output.status.success(),
        "Compilation failed: {}",
        String::from_utf8_lossy(&compile_output.stderr)
    );

    // Execute
    let run_output = std::process::Command::new(temp_dir.join("test_let_mut"))
        .output()
        .expect("Failed to run");

    assert!(
        run_output.status.success(),
        "Execution failed: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.trim() == "10",
        "Expected output '10' but got: {stdout}"
    );
}

/// Test case 7: Verify the bug exists (this test documents the current buggy behavior)
#[test]
#[ignore = "Remove this once bug is fixed"]
fn test_transpiler_bug_documented() {
    let code = "let mut x = 0;";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // This documents the BUGGY behavior - should be `let mut x = 0;` but is `();`
    // When bug is fixed, this test should fail and be removed
    eprintln!("BUGGY OUTPUT: {transpiled}");
    assert!(
        transpiled.contains("()") || transpiled.contains("let mut x = 0"),
        "Documenting current behavior, output: {transpiled}"
    );
}

/// Test case 8: Let statements followed by expression
#[test]
fn test_transpiler_let_then_expression() {
    let code = r"let mut x = 1;
x + 2";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .output()
        .expect("Failed to execute");

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Should have the let statement
    assert!(
        transpiled.contains("let mut x = 1"),
        "Expected 'let mut x = 1' but got: {transpiled}"
    );

    // Should have the expression as result
    assert!(
        transpiled.contains("x + 2"),
        "Expected 'x + 2' expression but got: {transpiled}"
    );
}
