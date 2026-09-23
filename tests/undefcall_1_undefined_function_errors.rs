#![allow(missing_docs)]
//! UNDEFCALL-1: calling an undefined function is a runtime error.
//!
//! Before the fix, `eval_function_call` turned every call to an undefined
//! identifier into an actor `Message` object, so `undefined_fn(1)` printed
//! nothing and `ruchy run` exited 0. Only PascalCase names are message
//! constructors (`e.send(Say(4))`); an undefined name starting with anything
//! else must fail with `Undefined function: <name>`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Write `code` to `main.ruchy` in a fresh temp dir; run `ruchy <args..> main.ruchy` there.
fn run_file(args: &[&str], code: &str) -> assert_cmd::assert::Assert {
    let temp = TempDir::new().expect("create temp dir");
    fs::write(temp.path().join("main.ruchy"), code).expect("write main.ruchy");
    ruchy_cmd()
        .current_dir(temp.path())
        .args(args)
        .arg("main.ruchy")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
}

#[test]
fn test_undefcall_1_top_level_undefined_call_errors() {
    run_file(&["run"], "undefined_fn(1)\n")
        .code(1)
        .stderr(predicate::str::contains("Undefined function: undefined_fn"));
}

#[test]
fn test_undefcall_1_undefined_call_in_main_errors() {
    run_file(&["run"], "fun main() {\n    nosuch()\n}\n")
        .code(1)
        .stderr(predicate::str::contains("Undefined function: nosuch"));
}

#[test]
fn test_undefcall_1_typo_of_defined_function_errors() {
    let code = "fun greet(name) {\n    println(\"hi \" + name)\n}\n\
                fun main() {\n    gret(\"bob\")\n}\n";
    run_file(&["run"], code)
        .code(1)
        .stdout(predicate::str::contains("hi bob").not())
        .stderr(predicate::str::contains("Undefined function: gret"));
}

#[test]
fn test_undefcall_1_undefined_call_errors_without_run_subcommand() {
    run_file(&[], "fun main() {\n    nosuch(1, 2)\n}\n")
        .code(1)
        .stderr(predicate::str::contains("Undefined function: nosuch"));
}

#[test]
fn test_undefcall_1_undefined_call_stops_later_statements() {
    run_file(
        &["run"],
        "println(\"before\")\nmissing_fn()\nprintln(\"after\")\n",
    )
    .code(1)
    .stdout(predicate::str::contains("before"))
    .stdout(predicate::str::contains("after").not())
    .stderr(predicate::str::contains("Undefined function: missing_fn"));
}

#[test]
fn test_undefcall_1_defined_functions_still_work() {
    let code = "fun double(x) {\n    x * 2\n}\n\
                fun main() {\n    println(double(21))\n}\n";
    run_file(&["run"], code)
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_undefcall_1_actor_message_constructor_still_works() {
    let code = "actor Echo {\n    count: i32 = 0\n    receive {\n        \
                Say(n: i32) => println(n * 10)\n    }\n}\n\
                fun main() {\n    let e = spawn Echo\n    e.send(Say(4))\n}\n";
    run_file(&["run"], code)
        .success()
        .stdout(predicate::str::contains("40"));
}

#[test]
fn test_undefcall_1_pascal_case_call_is_message_value() {
    run_file(&["run"], "let m = Increment(5)\nprintln(m)\n")
        .success()
        .stdout(predicate::str::contains("Increment"));
}
