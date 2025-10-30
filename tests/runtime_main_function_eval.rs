#![allow(missing_docs)]
// BUG: fn main() in -e mode returns <function> instead of executing
// EXTREME TDD: RED → GREEN → REFACTOR

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_red_main_function_should_execute() {
    // RED: fn main() should execute, not return function reference
    let code = "fn main() { println(42) }";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_baseline_top_level_println_works() {
    // BASELINE: println works at top level
    let code = "println(42)";

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}
