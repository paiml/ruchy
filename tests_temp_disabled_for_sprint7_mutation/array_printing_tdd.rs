//! TDD for array printing with proper Display formatting

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
fn test_simple_string_printing() {
    // GREEN phase - this now works after fixing Display formatting
    let code = r#"
let s = "hello"
println(s)
"#;

    let output = run_test_file(code).expect("String printing should work");
    assert_eq!(output.trim(), "hello"); // No quotes
}

#[test]
fn test_array_printing_in_rest_pattern() {
    // GREEN phase - array printing now works with Debug formatting
    let code = r#"
let arr = [1, 2, 3, 4, 5]
let [first, ...rest] = arr
println(rest)
"#;

    let output = run_test_file(code).expect("Array printing should work now");
    assert!(output.contains("[2, 3, 4, 5]")); // Debug format is acceptable for arrays
}

#[test]
fn test_integer_printing() {
    // GREEN phase - this should work
    let code = r#"
let x = 42
println(x)
"#;

    let output = run_test_file(code).expect("Integer printing should work");
    assert_eq!(output.trim(), "42");
}
