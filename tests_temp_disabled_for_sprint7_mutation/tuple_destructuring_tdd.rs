//! TDD for tuple destructuring - RED phase first

use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

fn run_test_file(code: &str) -> Result<String, String> {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
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
fn test_tuple_destructuring_simple() {
    let code = r#"
let tup = (1, "hello")
let (x, y) = tup
println(x)
println(y)
"#;

    let result = run_test_file(code).expect("Should compile and run");
    assert_eq!(result.trim(), "1\n\"hello\"");
}

#[test]
fn test_tuple_destructuring_three_elements() {
    let code = r#"
let tup = (42, "world", true)
let (a, b, c) = tup
println(a)
println(b) 
println(c)
"#;

    let result = run_test_file(code).expect("Should compile and run");
    assert_eq!(result.trim(), "42\n\"world\"\ntrue");
}

#[test]
fn test_tuple_destructuring_nested() {
    let code = r#"
let nested = ((1, 2), (3, 4))
let ((a, b), (c, d)) = nested
println(a)
println(b)
println(c)
println(d)
"#;

    let result = run_test_file(code).expect("Should compile and run");
    assert_eq!(result.trim(), "1\n2\n3\n4");
}
