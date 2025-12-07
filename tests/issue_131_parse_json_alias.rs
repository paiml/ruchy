// ISSUE-131: parse_json() returns Message type instead of parsed JSON object
//
// ROOT CAUSE (Five Whys):
// 1. Why does parse_json() return Message? → Not recognized as builtin function
// 2. Why not recognized? → Not registered in builtin_init.rs
// 3. Why not registered? → Only json_parse registered, parse_json alias missing
// 4. Why missing? → Original implementation only registered underscore version
// 5. Why no test? → No validation that both aliases work
//
// FIX: Add parse_json registration to builtin_init.rs (ONE LINE)
//
// EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

use predicates::prelude::*;

// ============================================================================
// RED PHASE: These tests MUST fail before fix
// ============================================================================

#[test]
#[ignore = "Issue #131: parse_json() not yet implemented"]
fn test_issue_131_01_parse_json_simple_object() {
    // RED: parse_json() should parse and allow field access
    let script = r#"
fun main() {
    let json_str = '{"name": "test", "value": 42}'
    let data = parse_json(json_str)
    println(data["name"])
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[ignore = "Issue #131: parse_json() not yet implemented"]
fn test_issue_131_02_parse_json_nested_access() {
    // RED: parse_json() should support nested field access
    let script = r#"
fun main() {
    let json_str = '{"user": {"name": "Alice", "age": 30}}'
    let data = parse_json(json_str)
    println(data["user"]["name"])
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
#[ignore = "Issue #131: parse_json() not yet implemented"]
fn test_issue_131_03_parse_json_bench_009_pattern() {
    // RED: BENCH-009 pattern - exactly as specified in benchmark
    let script = r#"
fun main() {
    let json_str = '{"users": [{"name": "Bob"}]}'
    let data = parse_json(json_str)
    println(data["users"][0]["name"])
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"));
}

#[test]
#[ignore = "Issue #131: json_parse() not yet implemented"]
fn test_issue_131_04_json_parse_still_works() {
    // GREEN: Verify json_parse() (underscore version) still works
    let script = r#"
fun main() {
    let json_str = '{"name": "test"}'
    let data = json_parse(json_str)
    println(data["name"])
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
#[ignore = "Issue #131: JSON parsing not yet implemented"]
fn test_issue_131_05_both_aliases_identical() {
    // GREEN: Both parse_json() and json_parse() should produce identical results
    let script_parse_json = r#"
fun main() {
    let data = parse_json('{"x": 1, "y": 2}')
    println(data["x"])
    println(data["y"])
}
"#;

    let script_json_parse = r#"
fun main() {
    let data = json_parse('{"x": 1, "y": 2}')
    println(data["x"])
    println(data["y"])
}
"#;

    let output1 = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script_parse_json)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output2 = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script_json_parse)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        output1, output2,
        "parse_json() and json_parse() should produce identical output"
    );
}

#[test]
fn test_issue_131_06_parse_json_not_message_type() {
    // RED: Verify parse_json() does NOT return Message type
    // Note: Ruchy uses double quotes for strings (single quotes are for characters)
    let script = r#"
fun main() {
    let data = parse_json("{\"name\": \"test\"}")
    println(data)
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Message").not());
}
