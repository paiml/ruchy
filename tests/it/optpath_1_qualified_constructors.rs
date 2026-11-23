#![allow(missing_docs)]

//! OPTPATH-1: the qualified constructors `Option::Some`, `Option::None`,
//! `Result::Ok` and `Result::Err` build exactly the same values as the
//! unqualified `Some`, `None`, `Ok` and `Err`.

use assert_cmd::Command;
use predicates::prelude::*;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::Value;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Evaluate `src` in a fresh interpreter.
fn value_of(src: &str) -> Value {
    let mut interp = Interpreter::new();
    let expr = Parser::new(src)
        .parse()
        .unwrap_or_else(|e| panic!("`{src}` did not parse: {e}"));
    interp
        .eval_expr(&expr)
        .unwrap_or_else(|e| panic!("`{src}` failed: {e}"))
}

fn assert_same_value(qualified: &str, unqualified: &str) {
    assert_eq!(
        value_of(qualified),
        value_of(unqualified),
        "{qualified} should be the same value as {unqualified}"
    );
}

#[test]
fn test_optpath_1_01_option_some_is_some() {
    assert_same_value("Option::Some(3)", "Some(3)");
    assert_same_value(r#"Option::Some("x")"#, r#"Some("x")"#);
}

#[test]
fn test_optpath_1_02_option_none_is_none() {
    assert_same_value("Option::None", "None");
}

#[test]
fn test_optpath_1_03_result_ok_is_ok() {
    assert_same_value("Result::Ok(1)", "Ok(1)");
}

#[test]
fn test_optpath_1_04_result_err_is_err() {
    assert_same_value(r#"Result::Err("bad")"#, r#"Err("bad")"#);
}

#[test]
fn test_optpath_1_05_match_on_qualified_values() {
    let src = r#"
fun unwrap_or_zero(o) {
    match o {
        Some(x) => x,
        None => 0,
    }
}
fun ok_or_minus_one(r) {
    match r {
        Ok(v) => v,
        Err(_) => -1,
    }
}
println(unwrap_or_zero(Option::Some(7)))
println(unwrap_or_zero(Option::None))
println(ok_or_minus_one(Result::Ok(5)))
println(ok_or_minus_one(Result::Err("no")))
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(src)
        .assert()
        .success()
        .stdout(predicate::eq("7\n0\n5\n-1\n"));
}

#[test]
fn test_optpath_1_06_printing_matches_unqualified() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"println(Option::Some(3)); println(Option::None); println(Result::Ok(1)); println(Result::Err("e"))"#)
        .assert()
        .success()
        .stdout(predicate::eq("Some(3)\nNone\nOk(1)\nErr(\"e\")\n"));
}

#[test]
fn test_optpath_1_07_qualified_arg_is_evaluated_once() {
    let src = r#"
let mut n = 0
fun bump() { n = n + 1; n }
let v = Option::Some(bump())
println(n)
"#;
    ruchy_cmd()
        .arg("-e")
        .arg(src)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("1\n"));
}

#[test]
fn test_optpath_1_08_other_undefined_qualified_calls_still_error() {
    ruchy_cmd()
        .arg("-e")
        .arg("Option::Nope(1)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Option::Nope"));
}
