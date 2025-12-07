#![allow(missing_docs)]
//! QUALITY-015: Fix linter incorrectly reporting used functions as "unused variable"
//!
//! GitHub Issue #15: <https://github.com/your-org/ruchy/issues/15>
//!
//! BUG: ruchy lint incorrectly reports used functions as "unused variable"
//! ROOT CAUSE: Functions defined with `VarType::Local` instead of `VarType::Function`
//! FIX: Add `VarType::Function` variant and exclude from unused checks

use predicates::prelude::*;
use proptest::prelude::*;
use std::fs;

/// Helper function to run ruchy lint on code and return the output
fn lint_code(code: &str, test_name: &str) -> assert_cmd::assert::Assert {
    let temp_file = format!("/tmp/quality_015_{test_name}.ruchy");
    fs::write(&temp_file, code).unwrap();

    let assert = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(&temp_file)
        .assert();

    let _ = fs::remove_file(&temp_file);
    assert
}

// ========== Section 1: Function Usage Detection ==========

/// RED: Test that used function is NOT flagged as "unused variable"
#[test]
fn test_quality_015_01_used_function_not_flagged() {
    let code = r#"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(5, 3)
    println("Result: {}", result)
}
"#;

    lint_code(code, "01")
        .success()
        .stdout(predicate::str::contains("unused variable: add").not())
        .stdout(predicate::str::contains("unused variable: main").not());
}

/// RED: Test that multiple used functions are NOT flagged
#[test]
fn test_quality_015_02_multiple_used_functions_not_flagged() {
    let code = r#"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun greet(name: str) {
    println("Hello, {}!", name)
}

fun main() {
    let result = add(5, 3)
    println("Result: {}", result)
    greet("World")
}
"#;

    lint_code(code, "02")
        .success()
        .stdout(predicate::str::contains("unused variable: add").not())
        .stdout(predicate::str::contains("unused variable: greet").not())
        .stdout(predicate::str::contains("unused variable: main").not());
}

/// RED: Test that `main()` is NOT flagged as "unused variable"
#[test]
fn test_quality_015_03_main_not_flagged_as_variable() {
    let code = r#"
fun main() {
    let x = 5
    println("x = {}", x)
}
"#;

    lint_code(code, "03")
        .success()
        .stdout(predicate::str::contains("unused variable: main").not());
}

// ========== Section 2: Mutual Function Calls ==========

/// RED: Test functions that call each other are NOT flagged
#[test]
fn test_quality_015_04_mutual_function_calls() {
    let code = r#"
fun helper(x: i32) -> i32 {
    x * 2
}

fun process(data: i32) -> i32 {
    helper(data) + 10
}

fun main() {
    let result = process(5)
    println("Result: {}", result)
}
"#;

    lint_code(code, "04")
        .success()
        .stdout(predicate::str::contains("unused variable: helper").not())
        .stdout(predicate::str::contains("unused variable: process").not())
        .stdout(predicate::str::contains("unused variable: main").not());
}

// ========== Section 3: Regression Tests (Unused Variables SHOULD Be Flagged) ==========

/// GREEN: Verify unused LOCAL variables are still correctly flagged
#[test]
fn test_quality_015_05_unused_local_variable_still_flagged() {
    let code = r#"
fun main() {
    let x = 5
    let y = 10
    println("Sum: {}", x + y)

    let unused = 42
}
"#;

    lint_code(code, "05")
        .success()
        .stdout(predicate::str::contains("unused variable: unused"));
}

/// GREEN: Verify unused variables in function bodies are still flagged
#[test]
fn test_quality_015_06_unused_variable_in_function_body() {
    let code = r#"
fun calculate(a: i32, b: i32) -> i32 {
    let temp = a + b
    let unused = 99
    temp
}

fun main() {
    let result = calculate(5, 3)
    println("Result: {}", result)
}
"#;

    lint_code(code, "06")
        .success()
        .stdout(predicate::str::contains("unused variable: unused"))
        .stdout(predicate::str::contains("unused variable: calculate").not())
        .stdout(predicate::str::contains("unused variable: main").not());
}

// ========== Section 4: Truly Unused Functions ==========

/// GREEN: Verify truly unused functions don't crash the linter
#[test]
fn test_quality_015_07_truly_unused_function() {
    let code = r#"
fun used_function() -> i32 {
    42
}

fun unused_function() -> i32 {
    99
}

fun main() {
    let result = used_function()
    println("Result: {}", result)
}
"#;

    lint_code(code, "07")
        .success()
        .stdout(predicate::str::contains("unused variable: used_function").not())
        .stdout(predicate::str::contains("unused variable: unused_function").not());
}

// ========== Section 5: GitHub Issue #15 Exact Reproduction ==========

/// RED: Exact reproduction from GitHub Issue #15 - simple case
#[test]
fn test_quality_015_08_github_issue_15_simple_case() {
    let code = r#"
fun main() {
    let x = 5
    let y = 10
    println("Sum: {}", x + y)

    let unused = 42
}
"#;

    lint_code(code, "08")
        .success()
        .stdout(predicate::str::contains("unused variable: unused"))
        .stdout(predicate::str::contains("unused variable: x").not())
        .stdout(predicate::str::contains("unused variable: y").not())
        .stdout(predicate::str::contains("unused variable: main").not());
}

/// RED: Exact reproduction from GitHub Issue #15 - function case
#[test]
fn test_quality_015_09_github_issue_15_function_case() {
    let code = r#"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun greet(name: str) {
    println("Hello, {}!", name)
}

fun main() {
    let result = add(5, 3)
    println("Result: {}", result)
    greet("World")
}
"#;

    lint_code(code, "09")
        .success()
        .stdout(predicate::str::contains("unused variable: add").not())
        .stdout(predicate::str::contains("unused variable: greet").not())
        .stdout(predicate::str::contains("unused variable: main").not())
        .stdout(predicate::str::contains("unused variable: result").not());
}

// ========== Section 6: Property-Based Tests (10K+ cases) ==========

/// Generate valid Ruchy function names (alphanumeric + underscore, starting with letter)
fn function_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}").expect("valid regex")
}

/// Generate simple function bodies (avoid syntax errors)
fn function_body_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("42".to_string()),
        Just("a + b".to_string()),
        Just("x * 2".to_string()),
        Just("\"hello\"".to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    /// Property: Used functions NEVER flagged as "unused variable" (10K cases)
    #[test]
    fn test_quality_015_10_property_used_functions_never_flagged(
        fn_name in function_name_strategy(),
        body in function_body_strategy()
    ) {
        // Skip reserved keywords
        if fn_name == "main" || fn_name == "if" || fn_name == "let" || fn_name == "fun" {
            return Ok(());
        }

        let code = format!(
            r#"
fun {fn_name}() -> i32 {{
    {body}
}}

fun main() {{
    let result = {fn_name}()
    println("Result: {{}}", result)
}}
"#
        );

        let temp_file = format!("/tmp/quality_015_prop_{fn_name}.ruchy");
        fs::write(&temp_file, &code).unwrap();

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("lint")
            .arg(&temp_file)
            .output()
            .unwrap();

        let _ = fs::remove_file(&temp_file);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Property: Function name should NEVER appear in "unused variable" warnings
        let unused_msg = format!("unused variable: {fn_name}");
        prop_assert!(
            !stdout.contains(&unused_msg) && !stderr.contains(&unused_msg),
            "Function '{}' was incorrectly flagged as unused variable\nStdout: {}\nStderr: {}",
            fn_name, stdout, stderr
        );
    }

    /// Property: Unused local variables ALWAYS flagged (regression check, 10K cases)
    #[test]
    #[ignore]
    fn test_quality_015_11_property_unused_locals_always_flagged(
        var_name in prop::string::string_regex("[a-z][a-z0-9_]{0,15}").expect("valid regex")
    ) {
        // Skip reserved keywords and common patterns
        if var_name == "main" || var_name == "if" || var_name == "let" || var_name == "fun" {
            return Ok(());
        }

        let code = format!(
            r#"
fun main() {{
    let x = 5
    let {var_name} = 42
    println("x = {{}}", x)
}}
"#
        );

        let temp_file = format!("/tmp/quality_015_prop_unused_{var_name}.ruchy");
        fs::write(&temp_file, &code).unwrap();

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("lint")
            .arg(&temp_file)
            .output()
            .unwrap();

        let _ = fs::remove_file(&temp_file);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Property: Unused local variable SHOULD be flagged
        let unused_msg = format!("unused variable: {var_name}");
        prop_assert!(
            stdout.contains(&unused_msg),
            "Unused variable '{}' was NOT flagged (regression)\nStdout: {}",
            var_name, stdout
        );
    }

    /// Property: Main function NEVER flagged regardless of body (10K cases)
    #[test]
    fn test_quality_015_12_property_main_never_flagged(
        body in function_body_strategy()
    ) {
        let code = format!(
            r#"
fun main() {{
    let x = {body}
    println("x = {{}}", x)
}}
"#
        );

        let temp_file = "/tmp/quality_015_prop_main.ruchy";
        fs::write(temp_file, &code).unwrap();

        let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
            .arg("lint")
            .arg(temp_file)
            .output()
            .unwrap();

        let _ = fs::remove_file(temp_file);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Property: main() should NEVER be flagged as unused variable
        prop_assert!(
            !stdout.contains("unused variable: main"),
            "main() was incorrectly flagged as unused\nStdout: {}",
            stdout
        );
    }
}
