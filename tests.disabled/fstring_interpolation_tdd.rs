use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

fn run_test_file(code: &str) -> String {
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

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_simple_fstring_interpolation() {
    let code = r#"
let name = "Alice"
let result = f"Hello {name}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("Hello Alice"),
        "Expected 'Hello Alice' but got: {}",
        output
    );
}

#[test]
fn test_fstring_with_expression() {
    let code = r#"
let x = 5
let y = 3
let result = f"Sum: {x + y}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("Sum: 8"),
        "Expected 'Sum: 8' but got: {}",
        output
    );
}

#[test]
fn test_fstring_multiple_interpolations() {
    let code = r#"
let score = 85
let grade = "B"
let result = f"Score {score} = Grade {grade}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("Score 85 = Grade B"),
        "Expected 'Score 85 = Grade B' but got: {}",
        output
    );
}

#[test]
fn test_fstring_with_method_call() {
    let code = r#"
let num = 42
let result = f"Number: {num.to_string()}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("Number: 42"),
        "Expected 'Number: 42' but got: {}",
        output
    );
}

#[test]
fn test_fstring_escaped_braces() {
    let code = r#"
let value = 10
let result = f"Literal {{braces}} and value: {value}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("Literal {braces} and value: 10"),
        "Expected escaped braces but got: {}",
        output
    );
}

#[test]
fn test_nested_fstring_expressions() {
    let code = r#"
let x = 5
let y = 3
let result = f"5! = {x * (x-1) * (x-2) * (x-3) * (x-4)}"
println(result)
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("5! = 120"),
        "Expected '5! = 120' but got: {}",
        output
    );
}

#[test]
fn test_fstring_in_println() {
    let code = r#"
let fact_5 = 120
println(f"5! = {fact_5}")
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("5! = 120"),
        "Expected '5! = 120' but got: {}",
        output
    );
}

#[test]
fn test_fstring_boolean_result() {
    let code = r#"
let fact_5 = 120
println(f"5! = 120: {fact_5 == 120}")
"#;
    let output = run_test_file(code);
    assert!(
        output.contains("5! = 120: true"),
        "Expected '5! = 120: true' but got: {}",
        output
    );
}
