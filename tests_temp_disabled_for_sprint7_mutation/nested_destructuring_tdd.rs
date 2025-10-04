//! TDD for nested destructuring compilation issues

use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

fn run_test_file(code: &str) -> Result<String, String> {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), code).expect("Failed to write test code");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--quiet",
            "--bin",
            "ruchy",
            "--",
            "run",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run ruchy");

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_simple_nested_arrays() {
    // GREEN phase - nested destructuring now works!
    let code = r#"
let nested = [[1, 2], [3, 4]]
let [[a, b], [c, d]] = nested
println(f"{a} {b} {c} {d}")
"#;

    let output = run_test_file(code).expect("Nested destructuring should work now");
    assert_eq!(output.trim(), "1 2 3 4");
}

#[test]
fn test_single_level_array() {
    // GREEN phase - this should already work
    let code = r#"
let arr = [1, 2]
let [a, b] = arr
println(f"{a} {b}")
"#;

    let output = run_test_file(code).expect("Single level destructuring should work");
    assert_eq!(output.trim(), "1 2");
}
