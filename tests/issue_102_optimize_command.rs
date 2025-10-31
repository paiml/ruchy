//! Issue #102: ruchy optimize not implemented
//!
//! Tests the `ruchy optimize` command for hardware-aware optimization analysis.
//!
//! Reference: https://github.com/paiml/ruchy/issues/102
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_simple_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "simple.ruchy",
        r#"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fun main() {
    println(factorial(5));
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Optimization Analysis"));
}

#[test]
fn test_issue_102_optimize_with_loops() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "loops.ruchy",
        r#"
fun sum_array(arr) {
    let total = 0;
    for i in 0..arr.len() {
        total = total + arr[i];
    }
    total
}

fun main() {
    let numbers = [1, 2, 3, 4, 5];
    println(sum_array(numbers));
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_multiple_functions() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "multi.ruchy",
        r#"
fun add(x, y) { x + y }
fun sub(x, y) { x - y }
fun mul(x, y) { x * y }
fun div(x, y) { x / y }

fun main() {
    println(add(1, 2));
    println(sub(5, 3));
    println(mul(4, 5));
    println(div(10, 2));
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .assert()
        .success();
}

// ============================================================================
// HARDWARE PROFILE TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_hardware_detect() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("detect")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hardware"));
}

#[test]
fn test_issue_102_optimize_hardware_intel() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("intel")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_hardware_amd() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("amd")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_hardware_arm() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("arm")
        .assert()
        .success();
}

// ============================================================================
// ANALYSIS DEPTH TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_depth_quick() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--depth")
        .arg("quick")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_depth_standard() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--depth")
        .arg("standard")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_depth_deep() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--depth")
        .arg("deep")
        .assert()
        .success();
}

// ============================================================================
// ANALYSIS FLAG TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_cache_analysis() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "cache.ruchy",
        r#"
fun process_matrix(matrix) {
    for row in matrix {
        for val in row {
            println(val);
        }
    }
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--cache")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache"));
}

#[test]
fn test_issue_102_optimize_branch_analysis() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "branches.ruchy",
        r#"
fun classify(x) {
    if x < 0 {
        "negative"
    } else if x == 0 {
        "zero"
    } else {
        "positive"
    }
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--branches")
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch"));
}

#[test]
fn test_issue_102_optimize_vectorization() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "vector.ruchy",
        r#"
fun dot_product(a, b) {
    let result = 0;
    for i in 0..a.len() {
        result = result + a[i] * b[i];
    }
    result
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--vectorization")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vectorization"));
}

#[test]
fn test_issue_102_optimize_abstractions() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "abstractions.ruchy",
        r#"
fun compose(f, g) {
    fun(x) { f(g(x)) }
}

fun square(x) { x * x }
fun double(x) { x * 2 }

fun main() {
    let f = compose(square, double);
    println(f(5));
}
"#,
    );

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--abstractions")
        .assert()
        .success()
        .stdout(predicate::str::contains("Abstraction"));
}

#[test]
fn test_issue_102_optimize_benchmark() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--benchmark")
        .assert()
        .success()
        .stdout(predicate::str::contains("Benchmark"));
}

// ============================================================================
// OUTPUT FORMAT TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_format_text() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--format")
        .arg("text")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_format_json() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");
    let output_file = temp.path().join("output.json");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Check JSON file was created
    assert!(output_file.exists(), "JSON output file should be created");

    // Verify it's valid JSON
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(
        content.contains("{") && content.contains("}"),
        "Output should be valid JSON"
    );
}

#[test]
fn test_issue_102_optimize_format_html() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");
    let output_file = temp.path().join("output.html");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--format")
        .arg("html")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Check HTML file was created
    assert!(output_file.exists(), "HTML output file should be created");

    // Verify it contains HTML tags
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(
        content.contains("<html") || content.contains("<!DOCTYPE"),
        "Output should be HTML"
    );
}

// ============================================================================
// OPTION TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_verbose() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Analysis").or(predicate::str::contains("Optimization")));
}

#[test]
fn test_issue_102_optimize_threshold() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--threshold")
        .arg("0.1")
        .assert()
        .success();
}

#[test]
fn test_issue_102_optimize_output_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");
    let output = temp.path().join("analysis.txt");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Output file should be created");
}

// ============================================================================
// COMBINED FLAGS TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_all_flags() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "comprehensive.ruchy",
        r#"
fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    println(fibonacci(10));
}
"#,
    );
    let output = temp.path().join("comprehensive.txt");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("detect")
        .arg("--depth")
        .arg("deep")
        .arg("--cache")
        .arg("--branches")
        .arg("--vectorization")
        .arg("--abstractions")
        .arg("--benchmark")
        .arg("--verbose")
        .arg("--threshold")
        .arg("0.05")
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Comprehensive analysis output should be created");
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_issue_102_optimize_missing_file() {
    ruchy_cmd()
        .arg("optimize")
        .arg("nonexistent_xyz_12345.ruchy")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn test_issue_102_optimize_invalid_hardware() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--hardware")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("hardware").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_102_optimize_invalid_depth() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--depth")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("depth").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_102_optimize_invalid_format() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.ruchy", "fun main() { println(42); }");

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .arg("--format")
        .arg("invalid_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("format").or(predicate::str::contains("invalid")));
}

#[test]
fn test_issue_102_optimize_syntax_error() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.ruchy", "fun bad( { }"); // Invalid syntax

    ruchy_cmd()
        .arg("optimize")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("parse")));
}
