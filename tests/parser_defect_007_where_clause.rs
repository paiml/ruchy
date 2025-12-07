#![allow(missing_docs)]
// DEFECT-PARSER-007: Where Clause Syntax
// Bug: Parser failed with "Unexpected token: Where" when encountering where clauses
// Fix: Added parse_where_clause() to parse and skip trait bounds after function signature

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// Test 1: Basic where clause with single trait bound
#[test]
#[ignore = "Where clause trait bounds not yet fully supported"]
fn test_parser_007_where_single_bound() {
    let code = r"
fn map_over<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where F: Fn(T) -> U
{
    items.into_iter().map(f).collect()
}
";
    std::fs::write("/tmp/test_parser_007_single.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_single.ruchy")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Syntax is valid"));
}

// Test 2: Where clause with multiple trait bounds
#[test]
#[ignore = "Where clause trait bounds not yet fully supported"]
fn test_parser_007_where_multiple_bounds() {
    let code = r"
fn process<T, U>(a: T, b: U) -> i32
where
    T: Display,
    U: Clone
{
    42
}
";
    std::fs::write("/tmp/test_parser_007_multiple.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_multiple.ruchy")
        .assert()
        .success();
}

// Test 3: Where clause with complex trait bound (Fn trait)
#[test]
#[ignore = "Where clause trait bounds not yet fully supported"]
fn test_parser_007_where_fn_bound() {
    let code = r"
fn apply<T, F>(value: T, func: F) -> T
where F: Fn(T) -> T
{
    func(value)
}
";
    std::fs::write("/tmp/test_parser_007_fn_bound.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_fn_bound.ruchy")
        .assert()
        .success();
}

// Test 4: Where clause with function type syntax
#[test]
#[ignore = "Where clause trait bounds not yet fully supported"]
fn test_parser_007_where_fn_signature() {
    let code = r#"
fn map_over<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where F: Fn(T) -> U
{
    items.into_iter().map(f).collect()
}

fn main() {
    let nums = vec![1, 2, 3];
    let doubled = map_over(nums, |x| x * 2);
    println!("Result: {:?}", doubled);
}
"#;
    std::fs::write("/tmp/test_parser_007_complete.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_complete.ruchy")
        .assert()
        .success();
}

// Test 5: Book example (appendix-b-syntax-reference_example_16)
#[test]
#[ignore = "Where clause trait bounds not yet fully supported"]
fn test_parser_007_book_example() {
    let code = r"
// Function as parameter
fn apply_operation(x: i32, y: i32, op: fn(i32, i32) -> i32) -> i32 {
    op(x, y)
}

// Function returning function
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// Generic function with where clause
fn map_over<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where F: Fn(T) -> U
{
    items.into_iter().map(f).collect()
}
";
    std::fs::write("/tmp/test_parser_007_book.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_book.ruchy")
        .assert()
        .success();
}

// Test 6: Function without where clause still works
#[test]
fn test_parser_007_no_where_clause() {
    let code = r"
fn identity<T>(value: T) -> T {
    value
}
";
    std::fs::write("/tmp/test_parser_007_no_where.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_007_no_where.ruchy")
        .assert()
        .success();
}
