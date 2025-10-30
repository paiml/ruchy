#![allow(missing_docs)]
// DEFECT-PARSER-006: impl Trait Return Types
// Bug: Parser failed with "Expected type" when parsing impl Trait return types
// Fix: Added Token::Impl handling in parse_type() + parse_impl_trait_type() function

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic impl Fn return type
#[test]
fn test_parser_006_impl_fn_return() {
    let code = r"
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
";
    std::fs::write("/tmp/test_parser_006_impl_fn.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_impl_fn.ruchy")
        .assert()
        .success();
}

// Test 2: impl FnOnce return type
#[test]
fn test_parser_006_impl_fn_once_return() {
    let code = r"
fn make_consumer(data: Vec<i32>) -> impl FnOnce() -> i32 {
    move || data.len() as i32
}
";
    std::fs::write("/tmp/test_parser_006_impl_fn_once.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_impl_fn_once.ruchy")
        .assert()
        .success();
}

// Test 3: impl FnMut return type
#[test]
fn test_parser_006_impl_fn_mut_return() {
    let code = r"
fn make_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}
";
    std::fs::write("/tmp/test_parser_006_impl_fn_mut.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_impl_fn_mut.ruchy")
        .assert()
        .success();
}

// Test 4: impl Fn with multiple parameters
#[test]
fn test_parser_006_impl_fn_multi_params() {
    let code = r"
fn make_op() -> impl Fn(i32, i32) -> i32 {
    move |x, y| x + y
}
";
    std::fs::write("/tmp/test_parser_006_impl_fn_multi.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_impl_fn_multi.ruchy")
        .assert()
        .success();
}

// Test 5: impl Fn as parameter (not just return type)
#[test]
fn test_parser_006_impl_fn_parameter() {
    let code = r"
fn apply_op(x: i32, y: i32, op: impl Fn(i32, i32) -> i32) -> i32 {
    op(x, y)
}
";
    std::fs::write("/tmp/test_parser_006_impl_fn_param.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_impl_fn_param.ruchy")
        .assert()
        .success();
}

// Test 6: Complete program with impl Trait
#[test]
fn test_parser_006_complete_program() {
    let code = r#"
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn main() {
    let add_five = make_adder(5);
    let result = add_five(10);
    println!("Result: {}", result);
}
"#;
    std::fs::write("/tmp/test_parser_006_complete.ruchy", code).unwrap();

    ruchy_cmd()
        .arg("check")
        .arg("/tmp/test_parser_006_complete.ruchy")
        .assert()
        .success();
}
