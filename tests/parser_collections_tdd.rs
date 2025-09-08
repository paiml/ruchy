//! TDD tests for parser collections functionality
//! Target: Improve coverage from 28.3% to 80%+

use std::process::Command;
use std::fs;
use tempfile::NamedTempFile;

fn run_test_file(code: &str) -> Result<String, String> {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), code).expect("Failed to write test code");
    
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--bin", "ruchy", "--", "run", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to run ruchy");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_array_destructuring() {
    let code = r#"
let arr = [1, 2, 3]
let [a, b, c] = arr
println(a)
println(b)
println(c)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "1\n2\n3");
}

#[test]
fn test_tuple_destructuring() {
    let code = r#"
let tup = (1, "hello", true)
let (x, y, z) = tup
println(x)
println(y)
println(z)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "1\nhello\ntrue");
}

#[test]
fn test_rest_pattern_array() {
    let code = r#"
let arr = [1, 2, 3, 4, 5]
let [first, ...rest] = arr
println(first)
println(rest)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert!(output.contains("1"));
    assert!(output.contains("[2, 3, 4, 5]"));
}

#[test]
fn test_nested_destructuring() {
    let code = r#"
let nested = [[1, 2], [3, 4]]
let [[a, b], [c, d]] = nested
println(f"{a} {b} {c} {d}")
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "1 2 3 4");
}

#[test]
fn test_object_destructuring() {
    let code = r#"
let obj = {name: "Alice", age: 30}
let {name, age} = obj
println(name)
println(age)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "Alice\n30");
}

#[test]
fn test_mixed_destructuring() {
    let code = r#"
let data = ([1, 2], {x: 10, y: 20})
let ([a, b], {x, y}) = data
println(f"{a} {b} {x} {y}")
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "1 2 10 20");
}

#[test]
fn test_array_spread_operator() {
    let code = r#"
let arr1 = [1, 2]
let arr2 = [3, 4]
let combined = [...arr1, ...arr2]
println(combined)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert!(output.contains("[1, 2, 3, 4]"));
}

#[test]
fn test_wildcard_pattern() {
    let code = r#"
let [_, second, _] = [1, 2, 3]
println(second)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "2");
}

#[test]
fn test_default_destructuring() {
    let code = r#"
let [a = 10, b = 20] = [1]
println(a)
println(b)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "1\n20");
}

#[test]
fn test_function_param_destructuring() {
    let code = r#"
fun process([x, y]) {
    return x + y
}
let result = process([5, 3])
println(result)
"#;
    let output = run_test_file(code).expect("Failed to run");
    assert_eq!(output.trim(), "8");
}