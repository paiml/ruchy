#![allow(missing_docs)]
//! RUNMAIN-1: `ruchy run` must call `main()` in a multi-item file.
//!
//! A file with more than one top-level item parses to a `Block`. The
//! interpreter evaluated that `Block` in a pushed scope and popped it
//! afterwards, so every function the file defined was gone before the
//! handler's trailing `main()` call, which then resolved to nothing and
//! printed nothing with exit 0.
//!
//! Semantics pinned here: after the top-level items run (once, in order),
//! `main()` is called once when the file defines it. A file whose top-level
//! code (anything other than a definition) contains a call to `main` drives
//! `main` itself, so the runner does not call it a second time. A call inside
//! another function's body does not count: that function only runs if called. A file without `main` only runs
//! its top-level code.

use std::fs;
use tempfile::TempDir;

/// Write `source` to `prog.ruchy` in a fresh temp dir and `ruchy run` it.
fn run_source(source: &str) -> assert_cmd::assert::Assert {
    let dir = TempDir::new().expect("create temp dir");
    fs::write(dir.path().join("prog.ruchy"), source).expect("write prog.ruchy");
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .current_dir(dir.path())
        .arg("run")
        .arg("prog.ruchy")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
}

#[test]
fn test_runmain_1_helper_then_main_prints_once() {
    run_source("fun d(a: i64) -> i64 { a }\nfun main() {\n  println(\"y\")\n}\n")
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_main_before_helper_prints_once() {
    run_source("fun main() {\n  println(\"y\")\n}\nfun d(a: i64) -> i64 { a }\n")
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_top_level_statement_runs_once_then_main() {
    run_source("fun d(a: i64) -> i64 { a }\nprintln(\"top\")\nfun main() {\n  println(\"y\")\n}\n")
        .success()
        .stdout("top\ny\n");
}

#[test]
fn test_runmain_1_explicit_trailing_main_call_runs_main_once() {
    run_source("fun d(a: i64) -> i64 { a }\nfun main() {\n  println(\"y\")\n}\nmain()\n")
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_single_main_with_explicit_call_runs_once() {
    run_source("fun main() { println(\"hi\") }\nmain()\n")
        .success()
        .stdout("hi\n");
}

#[test]
fn test_runmain_1_single_main_runs() {
    run_source("fun main() { println(\"hi\") }\n")
        .success()
        .stdout("hi\n");
}

#[test]
fn test_runmain_1_no_main_runs_top_level_code() {
    run_source("fun d(a: i64) -> i64 { a * 2 }\nprintln(d(21))\n")
        .success()
        .stdout("42\n");
}

#[test]
fn test_runmain_1_main_calls_helper_defined_before() {
    run_source("fun d(a: i64) -> i64 { a * 2 }\nfun main() {\n  println(d(21))\n}\n")
        .success()
        .stdout("42\n");
}

#[test]
fn test_runmain_1_main_calls_helper_defined_after() {
    run_source("fun main() {\n  println(d(21))\n}\nfun d(a: i64) -> i64 { a * 2 }\n")
        .success()
        .stdout("42\n");
}

#[test]
fn test_runmain_1_main_error_exits_nonzero() {
    run_source("fun d(a: i64) -> i64 { a }\nfun main() {\n  panic!(\"boom\")\n}\n").failure();
}

/// `ruchy <file>` (no subcommand) shares `ruchy run`'s program runner.
#[test]
fn test_runmain_1_default_file_path_calls_main() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("two.ruchy");
    std::fs::write(
        &file,
        "fun d(a: i64) -> i64 { a }\nfun main() {\n  println(\"y\")\n}\n",
    )
    .unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .current_dir(dir.path())
        .arg(&file)
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .success()
        .stdout("y\n");
}

const MAIN_Y: &str = "fun main() {\n  println(\"y\")\n  7\n}\n";

#[test]
fn test_runmain_1_let_bound_main_call_runs_main_once() {
    run_source(&format!("{MAIN_Y}let r = main()\n"))
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_main_call_as_argument_runs_main_once() {
    run_source(&format!("{MAIN_Y}println(main())\n"))
        .success()
        .stdout("y\n7\n");
}

#[test]
fn test_runmain_1_main_call_inside_top_level_if_runs_main_once() {
    run_source(&format!("{MAIN_Y}let c = true\nif c {{ main() }}\n"))
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_main_call_before_definition_in_block_runs_once() {
    run_source(&format!(
        "fun d(a: i64) -> i64 {{ a }}\n{MAIN_Y}{{\n  let v = main()\n}}\n"
    ))
    .success()
    .stdout("y\n");
}

#[test]
fn test_runmain_1_main_call_only_in_uncalled_helper_auto_calls_once() {
    run_source(&format!("fun helper() {{\n  main()\n}}\n{MAIN_Y}"))
        .success()
        .stdout("y\n");
}

#[test]
fn test_runmain_1_main_call_only_in_top_level_lambda_auto_calls_once() {
    run_source(&format!("{MAIN_Y}let f = || main()\n"))
        .success()
        .stdout("y\n");
}
