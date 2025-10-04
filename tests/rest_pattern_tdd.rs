//! TDD for rest patterns in destructuring

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
fn test_rest_pattern_simple() {
    // GREEN phase - rest patterns now work!
    let code = r#"
let arr = [1, 2, 3, 4, 5]
let [first, ...rest] = arr
println(first)
println(rest)
"#;

    let output = run_test_file(code).expect("Rest patterns should work now");
    assert!(output.contains("1"));
    assert!(output.contains("[2, 3, 4, 5]"));
}

#[test]
fn test_rest_pattern_named() {
    // GREEN phase - named rest patterns work!
    let code = r#"
let arr = [1, 2, 3]
let [head, ...tail] = arr
println(head)
println(tail)
"#;

    let output = run_test_file(code).expect("Named rest patterns should work now");
    assert!(output.contains("1"));
    assert!(output.contains("[2, 3]"));
}

#[test]
fn test_rest_pattern_empty() {
    // GREEN phase - rest with single element creates empty array
    let code = r#"
let arr = [42]
let [only, ...empty] = arr
println(only)
println(empty)
"#;

    let output = run_test_file(code).expect("Rest patterns with single element should work");
    assert!(output.contains("42"));
    assert!(output.contains("[]")); // Empty rest becomes empty array
}
