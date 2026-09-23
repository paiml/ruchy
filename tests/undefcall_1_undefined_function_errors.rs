#![allow(missing_docs)]
//! UNDEFCALL-1: calling an undefined function is a runtime error.
//!
//! Before the fix, `eval_function_call` turned every call to an undefined
//! identifier into an actor `Message` object, so `undefined_fn(1)` printed
//! nothing and `ruchy run` exited 0. An undefined callee is a message value
//! only in message position -- the argument of `send`/`ask`/`!`/`<?` -- whatever
//! its case (`e.send(Say(4))`, `acct.send(deposit(100))`); everywhere else it
//! fails with `Undefined function: <name>`, whatever its case.

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

/// A bank-account actor whose receive handlers are lower-case names.
const BANK_ACTOR: &str = "actor Bank {\n    balance: i32,\n\n    \
    receive deposit(amount: i32) {\n        self.balance = self.balance + amount;\n        \
    println(\"deposited \" + amount.to_string())\n    }\n\n    \
    receive get_balance() -> i32 {\n        self.balance\n    }\n}\n";

fn bank_program(main_body: &str) -> String {
    format!(
        "{BANK_ACTOR}\nfun main() {{\n    let acct = spawn Bank {{ balance: 0 }}\n{main_body}}}\n"
    )
}

#[test]
fn test_undefcall_1_lower_case_handler_receives_via_send() {
    run_file(&["run"], &bank_program("    acct.send(deposit(100))\n"))
        .success()
        .stdout(predicate::str::contains("deposited 100"));
}

#[test]
fn test_undefcall_1_lower_case_handler_receives_via_bang() {
    run_file(&["run"], &bank_program("    acct ! deposit(7)\n"))
        .success()
        .stdout(predicate::str::contains("deposited 7"));
}

#[test]
fn test_undefcall_1_lower_case_handler_receives_via_ask() {
    let body = "    acct.send(deposit(40))\n    let b = acct.ask(get_balance())\n    \
                println(\"balance=\" + b.to_string())\n";
    run_file(&["run"], &bank_program(body))
        .success()
        .stdout(predicate::str::contains("balance=40"));
}

#[test]
fn test_undefcall_1_lower_case_handler_receives_via_query_operator() {
    let body = "    acct ! deposit(9)\n    let b = acct <? get_balance()\n    \
                println(\"balance=\" + b.to_string())\n";
    run_file(&["run"], &bank_program(body))
        .success()
        .stdout(predicate::str::contains("balance=9"));
}

#[test]
fn test_undefcall_1_message_argument_still_evaluates_defined_calls() {
    let body = "    acct.send(deposit(twice(5)))\n";
    let code = format!("fun twice(x) {{\n    x * 2\n}}\n{}", bank_program(body));
    run_file(&["run"], &code)
        .success()
        .stdout(predicate::str::contains("deposited 10"));
}

#[test]
fn test_undefcall_1_undefined_call_nested_in_message_argument_errors() {
    run_file(
        &["run"],
        &bank_program("    acct.send(deposit(nosuch(5)))\n"),
    )
    .code(1)
    .stderr(predicate::str::contains("Undefined function: nosuch"));
}

#[test]
fn test_undefcall_1_pascal_case_call_outside_message_position_errors() {
    run_file(&["run"], "let m = Increment(5)\nprintln(m)\n")
        .code(1)
        .stderr(predicate::str::contains("Undefined function: Increment"));
}

#[test]
fn test_undefcall_1_pascal_case_typo_statement_errors() {
    run_file(
        &["run"],
        "fun main() {\n    Proces(3)\n    println(\"after\")\n}\n",
    )
    .code(1)
    .stdout(predicate::str::contains("after").not())
    .stderr(predicate::str::contains("Undefined function: Proces"));
}

#[test]
fn test_undefcall_1_result_constructors_are_not_undefined() {
    let code = "fun f(x) {\n    if x > 0 { Ok(x) } else { Err(\"neg\") }\n}\n\
                fun g(x) {\n    let v = f(x)?\n    Ok(v * 2)\n}\n\
                fun main() {\n    match f(3) {\n        Ok(v) => println(\"ok \" + v.to_string()),\n        \
                Err(e) => println(\"err \" + e),\n    }\n    match g(4) {\n        \
                Ok(v) => println(\"g \" + v.to_string()),\n        Err(e) => println(\"gerr \" + e),\n    }\n    \
                match g(-1) {\n        Ok(v) => println(\"g \" + v.to_string()),\n        \
                Err(e) => println(\"gerr \" + e),\n    }\n}\n";
    run_file(&["run"], code)
        .success()
        .stdout(predicate::str::contains("ok 3"))
        .stdout(predicate::str::contains("g 8"))
        .stdout(predicate::str::contains("gerr neg"));
}
