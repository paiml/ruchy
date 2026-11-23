// ISSUE-117: JSON parsing support
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// GitHub Issue: https://github.com/paiml/ruchy/issues/117

use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until JSON parsing is implemented
/// Current Error: No `JSON.parse()` or `JSON.stringify()` support
/// Expected: Parse JSON strings into objects like JavaScript

// ============================================================================
// TEST GROUP 1: Basic JSON Parsing
// ============================================================================

#[test]
#[ignore = "Issue #117: JSON.parse() not yet implemented"]
fn test_issue_117_parse_simple_object() {
    let code = r#"
        let json_str = '{"name": "Alice", "age": 30}';
        let obj = JSON.parse(json_str);
        println(obj["name"]);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
#[ignore = "Issue #117: JSON.parse() not yet implemented"]
fn test_issue_117_parse_array() {
    let code = r"
        let json_str = '[1, 2, 3, 4, 5]';
        let arr = JSON.parse(json_str);
        println(arr[0]);
        println(arr[4]);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("5"));
}

#[test]
#[ignore = "Issue #117: JSON.parse() not yet implemented"]
fn test_issue_117_parse_nested_object() {
    let code = r#"
        let json_str = '{"user": {"name": "Bob", "id": 123}}';
        let obj = JSON.parse(json_str);
        println(obj["user"]["name"]);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"));
}

#[test]
#[ignore = "Issue #117: JSON.parse() not yet implemented"]
fn test_issue_117_parse_numbers() {
    let code = r#"
        let json_str = '{"int": 42, "float": 3.14}';
        let obj = JSON.parse(json_str);
        println(obj["int"]);
        println(obj["float"]);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"))
        .stdout(predicate::str::contains("3.14"));
}

#[test]
#[ignore = "Issue #117: JSON.parse() not yet implemented"]
fn test_issue_117_parse_bool_null() {
    let code = r#"
        let json_str = '{"active": true, "value": null}';
        let obj = JSON.parse(json_str);
        println(obj["active"]);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// TEST GROUP 2: JSON Stringify
// ============================================================================

#[test]
fn test_issue_117_stringify_object() {
    let code = r#"
        let obj = {"name": "Charlie", "age": 25};
        let json_str = JSON.stringify(obj);
        println(json_str);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("name").and(predicate::str::contains("Charlie")));
}

#[test]
fn test_issue_117_stringify_array() {
    let code = r"
        let arr = vec![1, 2, 3];
        let json_str = JSON.stringify(arr);
        println(json_str);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("[1,2,3]").or(predicate::str::contains("[1, 2, 3]")));
}

// ============================================================================
// TEST GROUP 3: Round-trip (parse → stringify)
// ============================================================================

#[test]
#[ignore = "Issue #117: JSON.parse/stringify() not yet implemented"]
fn test_issue_117_roundtrip() {
    let code = r#"
        let json1 = '{"key": "value"}';
        let obj = JSON.parse(json1);
        let json2 = JSON.stringify(obj);
        println(json2);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("key").and(predicate::str::contains("value")));
}

// ============================================================================
// TEST GROUP 4: Error Handling
// ============================================================================

#[test]
fn test_issue_117_parse_invalid_json() {
    let code = r"
        let json_str = '{invalid json}';
        let obj = JSON.parse(json_str);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("-e").arg(code).assert().failure(); // Should fail with parse error
}

// ============================================================================
// TEST GROUP 5: Transpile Validation
// ============================================================================

#[test]
#[ignore = "Issue #117: JSON.parse/stringify() not yet implemented"]
fn test_issue_117_transpile_json_methods() {
    let code = r#"
        let obj = JSON.parse('{"test": true}');
        let str = JSON.stringify(obj);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("parse").and(predicate::str::contains("stringify")));
}

// ============================================================================
// PROPERTY TEST: Parse/stringify roundtrip preserves structure
// ============================================================================

#[test]
#[ignore = "Run with: cargo test --test issue_117_json_parsing -- --ignored"]
fn property_json_roundtrip_preserves_data() {
    use proptest::prelude::*;

    proptest!(|(name in "[a-z]{3,10}", age in 1..100u32)| {
        let json = format!(r#"{{"name":"{name}","age":{age}}}"#);

        let code = format!(r"
            let obj = JSON.parse('{json}');
            let result = JSON.stringify(obj);
            println(result);
        ");

        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        let output = cmd.arg("-e").arg(&code).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Roundtrip should preserve both fields
        assert!(stdout.contains(&name));
        assert!(stdout.contains(&age.to_string()));
    });
}
