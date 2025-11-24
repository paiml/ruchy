// OPTIMIZATION-001: High-level --optimize flag for NASA-grade binary optimization
// Tests for `ruchy compile --optimize <level>` command
// EXTREME TDD: RED phase - These tests WILL FAIL initially

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test 1: --optimize none (debug mode, opt-level=0)
/// Expected: Fastest compilation, largest binary, no optimizations
#[test]
fn test_optimization_001_01_optimize_none() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    let output_binary = temp_dir.path().join("simple_none");

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

    // OPTIMIZATION-001: Test ruchy compile --optimize none
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("none"); // NEW FLAG - will fail initially

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"))
        .stdout(predicate::str::contains("Binary size:"));

    // Verify binary was created
    assert!(output_binary.exists());

    // Verify binary is executable and runs correctly
    let run_output = std::process::Command::new(&output_binary).output().unwrap();
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("fibonacci(10) = 55"));
}

/// Test 2: --optimize balanced (opt-level=2, reasonable tradeoff)
/// Expected: Good performance, reasonable compile time, moderate binary size
#[test]
fn test_optimization_001_02_optimize_balanced() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple.ruchy");
    let output_binary = temp_dir.path().join("simple_balanced");

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

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("balanced");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"))
        .stdout(predicate::str::contains("Optimization level: balanced"));

    assert!(output_binary.exists());

    let run_output = std::process::Command::new(&output_binary).output().unwrap();
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("3 * 4 = 12"));
}

/// Test 3: --optimize aggressive (opt-level=3 + LTO + codegen-units=1)
/// Expected: Maximum performance, longer compile time, smaller binary
#[test]
fn test_optimization_001_03_optimize_aggressive() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("loop.ruchy");
    let output_binary = temp_dir.path().join("loop_aggressive");

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

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("aggressive");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"))
        .stdout(predicate::str::contains("Optimization level: aggressive"))
        .stdout(predicate::str::contains("LTO: fat"));

    assert!(output_binary.exists());

    let run_output = std::process::Command::new(&output_binary).output().unwrap();
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("Sum: 4950"));
}

/// Test 4: --optimize nasa (PGO + target-cpu=native + everything)
/// Expected: Absolute maximum performance, longest compile time, smallest binary
#[test]
fn test_optimization_001_04_optimize_nasa() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("benchmark.ruchy");
    let output_binary = temp_dir.path().join("benchmark_nasa");

    fs::write(
        &test_file,
        r#"
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(42);
    println!("square(42) = {}", result);
}
"#,
    )
    .unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("nasa");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"))
        .stdout(predicate::str::contains("Optimization level: nasa"))
        .stdout(predicate::str::contains("LTO: fat"))
        .stdout(predicate::str::contains("target-cpu: native"));

    assert!(output_binary.exists());

    let run_output = std::process::Command::new(&output_binary).output().unwrap();
    assert!(run_output.status.success());
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(stdout.contains("square(42) = 1764"));
}

/// Test 5: Binary size comparison (nasa < aggressive < balanced < none)
/// Expected: NASA produces smallest binary, none produces largest
#[test]
fn test_optimization_001_05_binary_size_comparison() {
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

    // Compile with each optimization level
    let binaries = vec![
        ("none", temp_dir.path().join("test_none")),
        ("balanced", temp_dir.path().join("test_balanced")),
        ("aggressive", temp_dir.path().join("test_aggressive")),
        ("nasa", temp_dir.path().join("test_nasa")),
    ];

    let mut sizes = Vec::new();

    for (level, output) in &binaries {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("compile")
            .arg(&test_file)
            .arg("--output")
            .arg(output)
            .arg("--optimize")
            .arg(level);

        cmd.assert().success();

        let size = fs::metadata(output).unwrap().len();
        sizes.push((level, size));
    }

    // Verify size ordering: aggressive/nasa < balanced < none
    // Note: NASA may be slightly larger than aggressive due to embed-bitcode,
    // but both should be smaller than balanced and none
    let none_size = sizes[0].1;
    let balanced_size = sizes[1].1;
    let aggressive_size = sizes[2].1;
    let nasa_size = sizes[3].1;

    // Both optimized binaries should be smaller than balanced
    assert!(
        aggressive_size < balanced_size,
        "Aggressive binary ({aggressive_size}) should be < balanced ({balanced_size})"
    );
    assert!(
        nasa_size < balanced_size,
        "NASA binary ({nasa_size}) should be < balanced ({balanced_size})"
    );
    assert!(
        balanced_size < none_size,
        "Balanced binary ({balanced_size}) should be < none ({none_size})"
    );
}

/// Test 6: Verbose output shows optimization details
/// Expected: Detailed information about optimization flags applied
#[test]
fn test_optimization_001_06_verbose_optimization_details() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("verbose.ruchy");
    let output_binary = temp_dir.path().join("verbose_test");

    fs::write(
        &test_file,
        r#"
fun main() {
    println!("Hello, World!");
}
"#,
    )
    .unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("aggressive")
        .arg("--verbose");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Optimization flags:"))
        .stdout(predicate::str::contains("lto=fat"))
        .stdout(predicate::str::contains("codegen-units=1"))
        .stdout(predicate::str::contains("strip=symbols"));
}

/// Test 7: Invalid optimization level error handling
/// Expected: Clear error message for invalid optimization level
#[test]
fn test_optimization_001_07_invalid_optimization_level() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("invalid.ruchy");

    fs::write(
        &test_file,
        r#"
fun main() {
    println!("Test");
}
"#,
    )
    .unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--optimize")
        .arg("invalid_level"); // Invalid optimization level

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "Invalid optimization level: invalid_level",
        ))
        .stderr(predicate::str::contains(
            "Valid levels: none, balanced, aggressive, nasa",
        ));
}

/// Test 8: JSON output with optimization metrics
/// Expected: Machine-readable JSON with optimization statistics
#[test]
fn test_optimization_001_08_json_output_with_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("json_test.ruchy");
    let output_binary = temp_dir.path().join("json_test");
    let json_output = temp_dir.path().join("optimization_report.json");

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

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_binary)
        .arg("--optimize")
        .arg("nasa")
        .arg("--json")
        .arg(&json_output);

    cmd.assert().success();

    // Check that JSON output file was created and contains valid JSON
    let json_content = fs::read_to_string(&json_output).unwrap();
    assert!(json_content.contains("\"optimization_level\": \"nasa\""));
    assert!(json_content.contains("\"binary_size\""));
    assert!(json_content.contains("\"compile_time_ms\""));
    assert!(json_content.contains("\"optimization_flags\""));
    assert!(json_content.contains("\"lto\": \"fat\""));
    assert!(json_content.contains("\"target_cpu\": \"native\""));
}
