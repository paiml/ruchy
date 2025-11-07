// PROFILING-001: Binary profiling for transpiled Rust code (Issue #138)
// Tests for `ruchy runtime --profile --binary` command
// EXTREME TDD: RED phase - These tests WILL FAIL initially

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test 1: Basic binary profiling with simple recursive fibonacci
/// Expected: Profile report shows function timings and call counts
#[test]
fn test_profiling_001_01_basic_binary_profiling() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("fibonacci.ruchy");

    // Create simple Ruchy file with recursive fibonacci
    fs::write(
        &test_file,
        r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    let result = fibonacci(10);
    println!("fibonacci(10) = {}", result);
}
"#,
    )
    .unwrap();

    // PROFILING-001: Test ruchy runtime --profile --binary fibonacci.ruchy
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary") // NEW FLAG - will fail initially
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("=== Binary Execution Profile ==="))
        .stdout(predicate::str::contains("fibonacci()")) // Function name
        .stdout(predicate::str::contains("main()")) // Function name
        .stdout(predicate::str::contains("ms")) // Timing in milliseconds
        .stdout(predicate::str::contains("calls")); // Call count
}

/// Test 2: Binary profiling with verbose output
/// Expected: Shows detailed timing breakdown and source file paths
#[test]
fn test_profiling_001_02_binary_profiling_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");

    fs::write(
        &test_file,
        r#"
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(5);
    println!("Result: {}", result);
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg("--verbose") // Verbose mode
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Binary Execution Profile"))
        .stdout(predicate::str::contains("Iterations:")) // Profiling iterations
        .stdout(predicate::str::contains("square()")) // Function timing
        .stdout(predicate::str::contains("File:")); // Source file path
}

/// Test 3: Binary profiling output format (text)
/// Expected: Human-readable text format with proper formatting
#[test]
fn test_profiling_001_03_binary_profiling_text_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("loop.ruchy");

    fs::write(
        &test_file,
        r#"
fun sum_numbers(n: i32) -> i32 {
    let mut total = 0;
    for i in 0..n {
        total = total + i;
    }
    total
}

fun main() {
    let result = sum_numbers(100);
    println!("Sum: {}", result);
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("=== Binary Execution Profile ==="))
        .stdout(predicate::str::contains("Function-level timings:"))
        .stdout(predicate::str::contains("sum_numbers()"))
        .stdout(predicate::str::contains("main()"))
        .stdout(predicate::str::contains("Memory:")) // Memory usage section
        .stdout(predicate::str::contains("Allocations:")); // Allocation tracking
}

/// Test 4: Binary profiling with iterations parameter
/// Expected: Runs multiple iterations and reports average/std deviation
#[test]
fn test_profiling_001_04_binary_profiling_iterations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("benchmark.ruchy");

    fs::write(
        &test_file,
        r#"
fun multiply(a: i32, b: i32) -> i32 {
    a * b
}

fun main() {
    let result = multiply(3, 4);
    println!("3 * 4 = {}", result);
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg("--iterations")
        .arg("100") // Run 100 iterations
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Iterations: 100"))
        .stdout(predicate::str::contains("multiply()"))
        .stdout(predicate::str::contains("ms")); // Average timing
}

/// Test 5: Binary profiling without --binary flag (interpreter profiling)
/// Expected: Falls back to interpreter profiling (existing behavior)
#[test]
fn test_profiling_001_05_interpreter_profiling_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    fs::write(
        &test_file,
        r#"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(2, 3);
    println!("Result: {}", result);
}
"#,
    )
    .unwrap();

    // Without --binary flag, should use interpreter profiling
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile") // No --binary flag
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Profiling")); // Some profiling output
}

/// Test 6: Binary profiling with compile errors
/// Expected: Graceful error message if transpilation/compilation fails
#[test]
fn test_profiling_001_06_binary_profiling_compile_error() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("broken.ruchy");

    // Invalid Ruchy code
    fs::write(
        &test_file,
        r"
fun broken() {
    let x = ;  // Syntax error
}
",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg(&test_file);

    // Should fail with clear error message
    cmd.assert().failure().stderr(
        predicate::str::contains("Parse error")
            .or(predicate::str::contains("Transpilation failed"))
            .or(predicate::str::contains("Compilation failed")),
    );
}

/// Test 7: Binary profiling shows optimization recommendations
/// Expected: Analysis suggests potential optimizations
#[test]
fn test_profiling_001_07_binary_profiling_recommendations() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("allocations.ruchy");

    fs::write(
        &test_file,
        r#"
fun create_vec(size: i32) -> Vec<i32> {
    let mut v = [];
    for i in 0..size {
        v.push(i);
    }
    v
}

fun main() {
    let v = create_vec(1000);
    println!("Created vector with {} elements", v.len());
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg(&test_file);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Recommendations:"))
        .stdout(predicate::str::contains("Memory:"));
}

/// Test 8: Binary profiling JSON output format
/// Expected: Machine-readable JSON format for CI/CD integration
#[test]
fn test_profiling_001_08_binary_profiling_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("json_test.ruchy");

    fs::write(
        &test_file,
        r#"
fun calculate() -> i32 {
    42
}

fun main() {
    let result = calculate();
    println!("Result: {}", result);
}
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("runtime")
        .arg("--profile")
        .arg("--binary")
        .arg("--output")
        .arg(temp_dir.path().join("profile.json"))
        .arg(&test_file);

    cmd.assert().success();

    // Check that JSON output file was created and contains valid JSON
    let json_output = fs::read_to_string(temp_dir.path().join("profile.json")).unwrap();
    assert!(json_output.contains("functions"));
    assert!(json_output.contains("timings"));
    assert!(json_output.contains("calculate"));
}
