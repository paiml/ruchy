#![allow(missing_docs)]
//! PARSER-074: Struct field visibility modifiers (pub(crate), pub(super))
//!
//! GitHub Issue: #57 (Part 3/3)
//! Bug: "Expected `RightParen`, found Crate" when parsing pub(crate) fields
//! Root Cause: Parser checks for `Token::Identifier("crate`") but lexer emits `Token::Crate`
//! Fix: Match `Token::Crate` and `Token::Super` in `parse_scoped_visibility()`

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic pub(crate) field
#[test]
fn test_parser_074_pub_crate_basic() {
    let code = r#"
struct BankAccount {
    pub owner: String,
    pub(crate) balance: f64
}

let account = BankAccount { owner: "Alice", balance: 100.0 }
println("{}", account.balance)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("100"));
}

// Test 2: pub(super) field visibility
#[test]
fn test_parser_074_pub_super_field() {
    let code = r#"
struct BankAccount {
    pub owner: String,
    pub(super) id: i32
}

let account = BankAccount { owner: "Alice", id: 123 }
println("{}", account.id)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("123"));
}

// Test 3: Mix of all visibility modifiers
#[test]
fn test_parser_074_mixed_visibility() {
    let code = r#"
struct User {
    pub name: String,
    pub(crate) email: String,
    pub(super) id: i32,
    password_hash: String
}

let user = User {
    name: "Alice",
    email: "alice@example.com",
    id: 42,
    password_hash: "hashed"
}
println("{}", user.name)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

// Test 4: Transpile mode - verify pub(crate) emitted correctly
#[test]
fn test_parser_074_transpile_pub_crate() {
    let code = r"
struct BankAccount {
    pub owner: String,
    pub(crate) balance: f64
}
";
    let temp_file = "/tmp/test_parser_074_transpile.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        // PARSER-074: prettyplease formats as "pub (crate)" with space
        .stdout(predicate::str::contains("pub (crate)"))
        .stdout(predicate::str::contains("pub owner"));
}

// Test 5: Transpile mode - verify pub(super) emitted correctly
#[test]
fn test_parser_074_transpile_pub_super() {
    let code = r"
struct User {
    pub(super) id: i32,
    name: String
}
";
    let temp_file = "/tmp/test_parser_074_transpile_super.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("transpile")
        .arg(temp_file)
        .assert()
        .success()
        // PARSER-074: prettyplease formats as "pub (super)" with space
        .stdout(predicate::str::contains("pub (super)"));
}

// Test 6: Multiple pub(crate) fields
#[test]
fn test_parser_074_multiple_pub_crate() {
    let code = r#"
struct Config {
    pub(crate) timeout: i32,
    pub(crate) retries: i32,
    pub(crate) verbose: bool
}

let config = Config { timeout: 30, retries: 3, verbose: true }
println("{}", config.timeout)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

// Test 7: pub(crate) in tuple struct - if supported
#[test]
#[ignore = May not be supported initially
fn test_parser_074_tuple_struct_visibility() {
    let code = r#"
struct Point(pub(crate) f64, pub(crate) f64)

let p = Point(3.0, 4.0)
println("{}", p.0)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

// Test 8: ruchy check mode - should validate visibility
#[test]
fn test_parser_074_check_mode() {
    let code = r"
struct BankAccount {
    pub(crate) balance: f64
}
";
    let temp_file = "/tmp/test_parser_074_check.ruchy";
    std::fs::write(temp_file, code).expect("Failed to write temp file");

    ruchy_cmd()
        .arg("check")
        .arg(temp_file)
        .assert()
        .success();
}

// Test 9: Nested struct with pub(crate)
#[test]
fn test_parser_074_nested_struct_pub_crate() {
    let code = r#"
struct Outer {
    pub(crate) inner: Inner
}

struct Inner {
    pub(crate) value: i32
}

let outer = Outer { inner: Inner { value: 42 } }
println("{}", outer.inner.value)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// Test 10: Regression - verify basic pub still works
#[test]
fn test_parser_074_regression_basic_pub() {
    let code = r#"
struct Point {
    pub x: f64,
    pub y: f64
}

let p = Point { x: 3.0, y: 4.0 }
println("{}", p.x)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}
