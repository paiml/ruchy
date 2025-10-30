//! REPL-005: Fix for loop () output in REPL (Issue #5)
//!
//! Root Cause: for/while loops return Value::Unit, which gets printed as "nil"
//! Expected: Unit values should NOT be printed in REPL (like they aren't in scripts)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test #1: for loop in REPL should NOT print nil/() after execution
#[test]
fn test_repl_005_for_loop_no_unit_output() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.write_stdin("for i in [1,2] { println(i) }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1\n2"))
        .stdout(predicate::str::contains("nil").not())
        .stdout(predicate::str::contains("()").not());
}

/// Test #2: while loop in REPL should NOT print nil/() after execution
#[test]
fn test_repl_005_while_loop_no_unit_output() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.write_stdin("let mut i = 0\nwhile i < 3 { println(i); i = i + 1 }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("0\n1\n2"))
        .stdout(predicate::str::contains("nil").not())
        .stdout(predicate::str::contains("()").not());
}

/// Test #3: if statement (returns Unit) should NOT print nil/()
#[test]
fn test_repl_005_if_statement_no_unit_output() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.write_stdin("if true { println(\"yes\") }\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"))
        .stdout(predicate::str::contains("nil").not())
        .stdout(predicate::str::contains("()").not());
}

/// Test #4: Variable assignment (returns Unit) should NOT print nil/()
#[test]
fn test_repl_005_let_binding_no_unit_output() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.write_stdin("let x = 42\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("nil").not())
        .stdout(predicate::str::contains("()").not());
}

/// Test #5: Expressions that return values SHOULD still print
#[test]
fn test_repl_005_value_expressions_do_print() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.write_stdin("2 + 2\n\"hello\"\ntrue\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("true"));
}

/// Test #6: Script execution should NOT print loop return values (baseline)
#[test]
fn test_repl_005_script_no_loop_output_baseline() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    writeln!(
        temp_file,
        "for i in [1,2] {{ println(i) }}\nprintln(\"done\")"
    )?;

    Command::cargo_bin("ruchy")?
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1\n2\ndone"))
        .stdout(predicate::str::contains("nil").not())
        .stdout(predicate::str::contains("()").not());

    Ok(())
}
