#![allow(missing_docs)]
// REGRESSION-076: Vec::new() causes infinite hang in v3.147.0
// GitHub Issue: https://github.com/paiml/ruchy/issues/76
//
// ROOT CAUSE: PARSER-091 fix made handle_colon_colon_operator() generate
// QualifiedName for ALL Module::identifier( patterns, including Vec::new().
// The runtime interpreter doesn't handle QualifiedName for standard library
// types like Vec, Box, HashMap, causing infinite loops.
//
// EXPECTED: Vec::new() should generate FieldAccess (existing behavior in v3.146.0)
// ACTUAL (v3.147.0): Vec::new() generates QualifiedName (breaks runtime)

use predicates::prelude::*;

/// Test Case 1: Minimal `Vec::new()` with while loop (8 lines)
/// This is the smallest reproduction case from Issue #76
#[test]
fn test_regression_076_minimal_vec_new() {
    let script = r#"
let mut vec = Vec::new();
let mut i = 0;
while i < 10 {
    vec.push(1.0);
    i += 1;
}
println!("Success: {} elements", vec.len());
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

/// Test Case 2: `Vec::push` in while loop (from logger test)
/// This worked in v3.146.0, hangs in v3.147.0
#[test]
fn test_regression_076_vec_push_loop() {
    let script = r#"
let mut messages = Vec::new();
let mut count = 0;
while count < 5 {
    messages.push("test");
    count += 1;
}
println!("Messages: {}", messages.len());
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Messages: 5"));
}

/// Test Case 3: Large Vec allocation (from vector-search test)
/// This worked in v3.146.0, hangs in v3.147.0 on test 7 (100 elements)
#[test]
fn test_regression_076_large_vec() {
    let script = r#"
let mut vec = Vec::new();
let mut i = 0;
while i < 100 {
    vec.push(i as f64);
    i += 1;
}
println!("Created vector with {} elements", vec.len());
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Created vector with 100 elements"));
}

/// Test Case 4: `Box::new()` should still work (should NOT generate `QualifiedName`)
/// This verifies that the fix doesn't break other standard library types
#[test]
fn test_regression_076_box_new_still_works() {
    let script = r#"
let boxed = Box::new(42);
println!("Boxed value: {}", boxed);
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Boxed value: 42"));
}

/// Test Case 5: `HashMap::new()` should still work
#[test]
fn test_regression_076_hashmap_new_still_works() {
    let script = r#"
let mut map = HashMap::new();
map.insert("key", "value");
println!("Map size: {}", map.len());
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Map size: 1"));
}

/// Test Case 6: `Command::new()` SHOULD generate `QualifiedName` (Issue #75)
/// This verifies that the PARSER-091 fix still works for Command
#[test]
#[ignore = "Still blocked by Issue #75 - Command runtime not implemented"]
fn test_regression_076_command_new_generates_qualifiedname() {
    let script = r#"
let cmd = Command::new("echo");
println!("Command created");
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("run")
        .write_stdin(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Command created"));
}
