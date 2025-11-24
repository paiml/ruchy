#![allow(missing_docs)]
// PARSER-087: Fix parser bug for methods with 2+ &str params + String return (GitHub Issue #68)
//
// RED Phase: These tests SHOULD FAIL with "Function parameters must be simple identifiers"
// GREEN Phase: After fix, these tests SHOULD PASS
//
// EXTREME TDD Protocol: RED → GREEN → REFACTOR
// Complexity Target: ≤10 (MANDATORY)

use predicates::prelude::*;

/// Test 1: Two &str params + bool return - SHOULD WORK (baseline)
/// This case currently works, so it's a control test
#[test]
fn test_parser_087_01_two_str_refs_bool_return() {
    let code = r"
impl ConfigManager {
    fun test_two_refs_bool(&self, key: &str, value: &str) -> bool {
        return true
    }
}
";

    let temp_file = "/tmp/test_parser_087_01.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}

/// Test 2: Two &str params + String return - SHOULD FAIL (BUG), THEN PASS (FIX)
/// RED: This is the EXACT bug from GitHub Issue #68
/// GREEN: After fix, this SHOULD PASS
#[test]
fn test_parser_087_02_two_str_refs_string_return() {
    let code = r#"
impl ConfigManager {
    fun test_two_refs_string(&self, key: &str, default: &str) -> String {
        return "test".to_string()
    }
}
"#;

    let temp_file = "/tmp/test_parser_087_02.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // RED: ruchy check should FAIL with "Function parameters must be simple identifiers"
    // GREEN: After fix, ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}

/// Test 3: One &str param + String return - SHOULD WORK (baseline)
#[test]
fn test_parser_087_03_one_str_ref_string_return() {
    let code = r#"
impl ConfigManager {
    fun test_one_ref_string(&self, key: &str) -> String {
        return "test".to_string()
    }
}
"#;

    let temp_file = "/tmp/test_parser_087_03.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}

/// Test 4: Three &str params + String return - SHOULD FAIL (BUG), THEN PASS (FIX)
/// This tests the bug with 3+ reference parameters
#[test]
fn test_parser_087_04_three_str_refs_string_return() {
    let code = r"
impl ConfigManager {
    fun get_config(&self, section: &str, key: &str, default: &str) -> String {
        return default.to_string()
    }
}
";

    let temp_file = "/tmp/test_parser_087_04.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // RED: ruchy check should FAIL
    // GREEN: After fix, ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}

/// Test 5: Mix of types with multiple &str params - SHOULD FAIL (BUG), THEN PASS (FIX)
#[test]
fn test_parser_087_05_mixed_types_with_str_refs() {
    let code = r#"
impl ConfigManager {
    fun complex_method(&self, name: &str, age: i32, city: &str) -> String {
        return name.to_string() + " " + city
    }
}
"#;

    let temp_file = "/tmp/test_parser_087_05.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // RED: ruchy check should FAIL
    // GREEN: After fix, ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}

/// Test 6: Regular functions (non-impl) with multiple &str params
#[test]
fn test_parser_087_06_regular_function_multiple_str_refs() {
    let code = r"
fun join_strings(left: &str, right: &str, separator: &str) -> String {
    return left.to_string() + separator + right
}
";

    let temp_file = "/tmp/test_parser_087_06.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write test file");

    // RED: ruchy check should FAIL
    // GREEN: After fix, ruchy check should PASS
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax is valid"));

    std::fs::remove_file(temp_file).ok();
}
