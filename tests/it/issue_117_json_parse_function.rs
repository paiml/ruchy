// Issue #117: JSON plain function API (parse_json, stringify_json)
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// Blocking: BENCH-009 (JSON parsing benchmark)
//
// ROOT CAUSE: Benchmarks expect parse_json() function, but only JSON.parse() method was implemented
// FIX: Register "parse_json" and "stringify_json" as builtin functions

use predicates::prelude::*;

#[test]
#[ignore = "Issue #117: parse_json() not yet implemented"]
fn test_issue_117_parse_json_simple_object() {
    // RED: This test WILL FAIL until parse_json() is registered as builtin
    let code = r#"
        let obj = parse_json('{"name": "Alice", "age": 30}')
        println(obj["name"])
    "#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
#[ignore = "Issue #117: parse_json() not yet implemented"]
fn test_issue_117_parse_json_array() {
    // RED: Array parsing
    let code = r"
        let arr = parse_json('[1, 2, 3]')
        println(arr[1])
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
#[ignore = "Issue #117: stringify_json() not yet implemented"]
fn test_issue_117_stringify_json_object() {
    // RED: This test WILL FAIL until stringify_json() is registered
    let code = r#"
        let obj = {"name": "Bob", "score": 95}
        let json = stringify_json(obj)
        println(json)
    "#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
#[ignore = "Issue #117: stringify_json() not yet implemented"]
fn test_issue_117_stringify_json_array() {
    // RED: Array stringification
    let code = r"
        let arr = [10, 20, 30]
        let json = stringify_json(arr)
        println(json)
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("[10,20,30]"));
}

#[test]
#[ignore = "Issue #117: JSON parsing not yet implemented"]
fn test_issue_117_roundtrip() {
    // RED: parse → modify → stringify roundtrip
    let code = r#"
        let obj = parse_json('{"count": 5}')
        let json = stringify_json(obj)
        println(json)
    "#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("count"))
        .stdout(predicate::str::contains("5"));
}

#[test]
#[ignore = "Issue #117: Nested array index access parsing bug"]
fn test_issue_117_nested_object() {
    // RED: Nested object access (BENCH-009 pattern)
    let code = r#"
        let data = parse_json('{"users": [{"name": "Alice"}]}')
        println(data["users"][0]["name"])
    "#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}
