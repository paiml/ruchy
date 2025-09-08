use std::process::Command;
use std::fs;
use tempfile::NamedTempFile;

fn run_test_file(code: &str) -> String {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), code).expect("Failed to write test code");
    
    let output = Command::new("cargo")
        .args(&["run", "--quiet", "--bin", "ruchy", "--", "run", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to run ruchy");
    
    if !output.status.success() {
        panic!("Compilation failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_multiple_statements_with_semicolons() {
    let code = r#"
let x = 5
println(x)
let y = 10
println(y)
"#;
    let output = run_test_file(code);
    assert!(output.contains("5\n10"), "Expected '5\\n10' but got: {}", output);
}

#[test]
fn test_function_with_multiple_println() {
    let code = r#"
fun factorial(n) {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}

let fact_5 = factorial(5)
println(f"5! = {fact_5}")
let fact_0 = factorial(0)
println(f"0! = {fact_0}")
"#;
    let output = run_test_file(code);
    assert!(output.contains("5! = 120"), "Expected '5! = 120' but got: {}", output);
    assert!(output.contains("0! = 1"), "Expected '0! = 1' but got: {}", output);
}

#[test]
fn test_consecutive_let_statements() {
    let code = r#"
let a = 1
let b = 2
let c = 3
println(a)
println(b)
println(c)
"#;
    let output = run_test_file(code);
    assert_eq!(output.trim(), "1\n2\n3", "Expected '1\\n2\\n3' but got: {}", output);
}

#[test]
fn test_mixed_statements_and_expressions() {
    let code = r#"
let x = 10
x + 5
let y = 20
println(y)
"#;
    let output = run_test_file(code);
    assert!(output.contains("20"), "Expected '20' but got: {}", output);
}