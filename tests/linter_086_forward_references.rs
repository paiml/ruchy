#![allow(missing_docs)]
// LINTER-086: Forward Reference Resolution (GitHub Issue #69)
// Bug: Linter reports "undefined variable" for functions defined later in file
// Expected: Two-pass analysis should resolve forward references
// Status: RED phase - This test SHOULD FAIL until GREEN phase

use predicates::prelude::*;

/// Test forward function reference (calling function defined later)
/// RED: This SHOULD FAIL with "undefined variable: `helper_function`"
/// GREEN: After two-pass implementation, this SHOULD PASS
#[test]
fn test_linter_086_01_forward_function_reference() {
    // Create minimal reproduction from GitHub Issue #69
    let code = r#"
fun main() {
    let result = helper_function()
    println!("Result: " + result.to_string())
}

fun helper_function() -> i32 {
    return 42
}
"#;

    // Write to temp file
    let temp_file = "/tmp/test_linter_086_01.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // ruchy check should PASS (code is valid)
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();

    // ruchy run should PASS (code executes correctly)
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicates::str::contains("Result: 42"));

    // RED: ruchy lint should FAIL with "undefined variable: helper_function"
    // GREEN: After fix, ruchy lint should PASS (no errors)
    let result = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(temp_file)
        .assert();

    // EXTREME TDD: This assertion will FAIL in RED phase
    // After two-pass implementation (GREEN phase), it will PASS
    result.success().stdout(
        predicates::str::contains("Summary: 0 Errors")
            .or(predicates::str::contains("No issues found")),
    );
}

/// Test mutual recursion (functions calling each other)
/// RED: This SHOULD FAIL (both functions reported as undefined)
/// GREEN: After two-pass implementation, this SHOULD PASS
#[test]
fn test_linter_086_02_mutual_recursion() {
    let code = r#"
fun is_even(n: i32) -> bool {
    if n == 0 {
        return true
    } else {
        return is_odd(n - 1)
    }
}

fun is_odd(n: i32) -> bool {
    if n == 0 {
        return false
    } else {
        return is_even(n - 1)
    }
}

fun main() {
    println!("4 is even: " + is_even(4).to_string())
    println!("5 is odd: " + is_odd(5).to_string())
}
"#;

    let temp_file = "/tmp/test_linter_086_02.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();

    // RED: ruchy lint should FAIL (reports is_odd undefined in is_even)
    // GREEN: After two-pass fix, should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(temp_file)
        .assert()
        .success();
}

/// Test function defined after main (standard Ruchy pattern)
/// RED: This SHOULD FAIL
/// GREEN: After two-pass implementation, this SHOULD PASS
#[test]
fn test_linter_086_03_helper_after_main() {
    let code = r#"
fun main() {
    let x = calculate(10, 20)
    let y = format_result(x)
    println!(y)
}

fun calculate(a: i32, b: i32) -> i32 {
    return a + b
}

fun format_result(value: i32) -> String {
    return "Result: " + value.to_string()
}
"#;

    let temp_file = "/tmp/test_linter_086_03.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // ruchy check and run should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();

    // RED: lint should FAIL (calculate and format_result undefined)
    // GREEN: After fix, should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(temp_file)
        .assert()
        .success();
}

/// Test GitHub Issue #69 exact reproduction
#[test]
fn test_linter_086_04_github_issue_69_exact() {
    // Exact code from GitHub Issue #69
    let code = r#"
// Minimal reproduction for ruchy lint false positives
// Bug: lint reports "undefined variable" for forward-referenced functions

fun main() {
    // Test 1: Forward reference to function defined later
    let result = helper_function()
    println!("Result: " + result.to_string())

    // Test 2: Built-in vec! macro
    let items = vec!["item1", "item2", "item3"]
    println!("Items count: " + items.len().to_string())
}

fun helper_function() -> i32 {
    return 42
}
"#;

    let temp_file = "/tmp/test_linter_086_04.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // Per Issue #69: check and run should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicates::str::contains("Syntax is valid").or(predicates::str::contains("✓")));

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicates::str::contains("Result: 42"))
        .stdout(predicates::str::contains("Items count: 3"));

    // RED: Per Issue #69, this FAILS with "undefined variable: helper_function"
    // GREEN: After two-pass fix, should report 0 Errors
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("lint")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(
            predicates::str::contains("0 Errors").or(predicates::str::contains("No issues found")),
        );
}
