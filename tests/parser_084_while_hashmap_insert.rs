#![allow(missing_docs)]
// PARSER-084: Test for Issue #67 Sub-issue #1 - while loop + HashMap.insert() parser bug
//
// ROOT CAUSE: Parser error "Expected RightBrace, found Let" at line beyond EOF
// SYMPTOM: Cannot use HashMap.insert() inside while loops
// IMPACT: Blocks ubuntu-config-scripts Ruchy conversion (RUCHY-002 GREEN)
//
// EXTREME TDD PROTOCOL:
// ✅ RED Phase: This test file (minimal reproduction)
// ⏸️ GREEN Phase: Fix parser to handle HashMap.insert() in while loops
// ⏸️ REFACTOR Phase: Clean up parser code if needed
// ⏸️ VERIFY Phase: All tests pass, no regressions

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_parser_084_while_loop_with_hashmap_insert() {
    // Minimal reproduction from Issue #67
    let code = r#"use std::collections::HashMap;

fun parse_args(args: Vec<String>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            let key_part = &arg[2..];
            parsed.insert(key_part.to_string(), String::from("value"));
        }

        i += 1;
    }

    parsed
}

fun main() {
    let test = vec![String::from("--test")];
    let result = parse_args(test);
    println("Done");
}
"#;

    let temp_file = PathBuf::from("/tmp/test_parser_084_while_hashmap.ruchy");
    fs::write(&temp_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success(); // Should parse successfully

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_parser_084_simplified_while_hashmap_insert() {
    // Simplified version to isolate the issue
    let code = r#"use std::collections::HashMap;

fun test_insert() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut i = 0;

    while i < 3 {
        map.insert(String::from("key"), String::from("value"));
        i += 1;
    }

    map
}

fun main() {
    let result = test_insert();
    println("Done");
}
"#;

    let temp_file = PathBuf::from("/tmp/test_parser_084_simplified.ruchy");
    fs::write(&temp_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success(); // Should parse successfully

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_parser_084_while_with_method_call() {
    // Test that method calls work in while loops (baseline)
    let code = r#"fun test_method_call() {
    let mut i = 0;
    let mut s = String::from("test");

    while i < 3 {
        s.push_str("!");
        i += 1;
    }
}

fun main() {
    test_method_call();
    println("Done");
}
"#;

    let temp_file = PathBuf::from("/tmp/test_parser_084_method_call.ruchy");
    fs::write(&temp_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success(); // Should parse successfully

    let _ = fs::remove_file(&temp_file);
}

#[test]
fn test_parser_084_for_loop_with_hashmap_insert() {
    // Test that HashMap.insert() works in for loops (control group)
    let code = r#"use std::collections::HashMap;

fun test_for_insert() -> HashMap<i32, String> {
    let mut map = HashMap::new();

    for i in 0..3 {
        map.insert(i, String::from("value"));
    }

    map
}

fun main() {
    let result = test_for_insert();
    println("Done");
}
"#;

    let temp_file = PathBuf::from("/tmp/test_parser_084_for_loop.ruchy");
    fs::write(&temp_file, code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("check")
        .arg(&temp_file)
        .assert()
        .success(); // Should parse successfully

    let _ = fs::remove_file(&temp_file);
}
