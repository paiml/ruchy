/// ISSUE-140: `time_micros()` in ruchy compile command
///
/// This test verifies that `ruchy compile` correctly transpiles `time_micros()`
/// to `SystemTime` API, not raw `time_micros()` function calls
use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_issue_140_ruchy_compile_time_micros_basic() {
    let temp_file = NamedTempFile::new().unwrap();
    let ruchy_code = r#"
fun main() {
    let t = time_micros()
    println(f"TIME: {t}")
}
"#;
    fs::write(&temp_file, ruchy_code).unwrap();

    // Test: ruchy compile should successfully compile code with time_micros()
    let output_binary = format!("{}_binary", temp_file.path().display());

    let result = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(temp_file.path())
        .arg("-o")
        .arg(&output_binary)
        .assert();

    // Should compile successfully (not fail with "cannot find function `time_micros`")
    result.success();

    // Clean up
    let _ = fs::remove_file(&output_binary);
}

#[test]
fn test_issue_140_ruchy_compile_time_micros_benchmark_pattern() {
    // This is the exact pattern from Issue #140
    let temp_file = NamedTempFile::new().unwrap();
    let ruchy_code = r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}

fun main() {
    let t0 = time_micros()
    let n = 35
    let t1 = time_micros()
    let result = fibonacci(n)
    let t2 = time_micros()

    println(f"STARTUP_TIME_US: {t1 - t0}")
    println(f"COMPUTE_TIME_US: {t2 - t1}")
    println(f"RESULT: {result}")
}
"#;
    fs::write(&temp_file, ruchy_code).unwrap();

    let output_binary = format!("{}_benchmark", temp_file.path().display());

    // Test: Should compile successfully
    let result = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(temp_file.path())
        .arg("-o")
        .arg(&output_binary)
        .assert();

    result.success();

    // Clean up
    let _ = fs::remove_file(&output_binary);
}

#[test]
fn test_issue_140_ruchy_compile_multiple_time_micros() {
    let temp_file = NamedTempFile::new().unwrap();
    let ruchy_code = r#"
fun main() {
    let t1 = time_micros()
    let work1 = 42 + 42
    let t2 = time_micros()
    let work2 = 100 * 100
    let t3 = time_micros()

    let duration1 = t2 - t1
    let duration2 = t3 - t2
    println(f"Duration 1: {duration1}")
    println(f"Duration 2: {duration2}")
}
"#;
    fs::write(&temp_file, ruchy_code).unwrap();

    let output_binary = format!("{}_multiple", temp_file.path().display());

    let result = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("compile")
        .arg(temp_file.path())
        .arg("-o")
        .arg(&output_binary)
        .assert();

    result.success();

    // Clean up
    let _ = fs::remove_file(&output_binary);
}
