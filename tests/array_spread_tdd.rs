//! TDD for array spread operator support

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
fn test_simple_array_spread() {
    // GREEN phase - array spread now works!
    let code = r#"
let arr1 = [1, 2]
let arr2 = [3, 4]
let combined = [...arr1, ...arr2]
println(combined)
"#;

    let output = run_test_file(code).expect("Array spread should work now");
    assert!(output.contains("[1, 2, 3, 4]"));
}

#[test]
fn test_regular_array_literal() {
    // GREEN phase - this should already work
    let code = r#"
let arr = [1, 2, 3, 4]
println(arr)
"#;

    let output = run_test_file(code).expect("Regular array literals should work");
    assert!(output.contains("[1, 2, 3, 4]"));
}
