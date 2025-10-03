// Layer 1: Fast Non-Interactive CLI Tests (assert_cmd)
//
// Per REPL testing spec: "Run FIRST (fastest feedback)"
// Target: <2s runtime for 50+ tests
// Uses stdin redirection (batch mode) for speed
//
// Critical Test Cases (Must Pass Before v0.1):
// - [ ] Batch mode evaluates expressions
// - [ ] Syntax errors exit non-zero
// - [ ] --help shows usage
// - [ ] --version shows version
// - [ ] Stdin EOF exits cleanly

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to create ruchy REPL command
fn repl_cmd() -> Command {
    Command::cargo_bin("ruchy").unwrap()
}

#[test]
fn cli_batch_mode_evaluates_integer() {
    repl_cmd()
        .arg("repl")
        .write_stdin("42\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn cli_batch_mode_evaluates_arithmetic() {
    repl_cmd()
        .arg("repl")
        .write_stdin("2 + 2\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn cli_batch_mode_evaluates_string() {
    repl_cmd()
        .arg("repl")
        .write_stdin("\"hello world\"\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn cli_batch_mode_evaluates_boolean() {
    repl_cmd()
        .arg("repl")
        .write_stdin("true\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn cli_syntax_error_continues_repl() {
    // Incomplete expressions may be handled as multiline - test with actual syntax error
    repl_cmd()
        .arg("repl")
        .write_stdin("let\n42\n:quit\n")
        .assert()
        .success() // REPL should still exit successfully
        .stdout(predicate::str::contains("42")); // Should process next line after error
}

#[test]
fn cli_help_flag_shows_usage() {
    repl_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("USAGE")));
}

#[test]
fn cli_version_flag_shows_version() {
    repl_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn cli_stdin_eof_exits_cleanly() {
    repl_cmd()
        .arg("repl")
        .write_stdin("") // Empty stdin (EOF)
        .assert()
        .success();
}

#[test]
fn cli_quit_command_exits_cleanly() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":quit\n")
        .assert()
        .success();
}

#[test]
fn cli_exit_command_exits_cleanly() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":exit\n")
        .assert()
        .success();
}

#[test]
fn cli_q_command_exits_cleanly() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":q\n")
        .assert()
        .success();
}

#[test]
fn cli_multiple_expressions() {
    repl_cmd()
        .arg("repl")
        .write_stdin("1 + 1\n2 + 2\n3 + 3\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("6"));
}

#[test]
fn cli_variable_binding() {
    repl_cmd()
        .arg("repl")
        .write_stdin("let x = 10\nx\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn cli_function_definition_and_call() {
    repl_cmd()
        .arg("repl")
        .write_stdin("fun add(a, b) { a + b }\nadd(2, 3)\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn cli_type_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":type 42\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integer").or(predicate::str::contains("Type")));
}

#[test]
fn cli_help_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":help\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commands").or(predicate::str::contains("help")));
}

#[test]
fn cli_vars_command_empty() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":vars\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No variables").or(predicate::str::contains("defined")));
}

#[test]
fn cli_vars_command_with_bindings() {
    repl_cmd()
        .arg("repl")
        .write_stdin("let x = 42\n:vars\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("x"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn cli_env_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":env\n:quit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Environment")
                .or(predicate::str::contains("Mode"))
                .or(predicate::str::contains("Variables")),
        );
}

#[test]
fn cli_mode_command_show_current() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":mode\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("mode").or(predicate::str::contains("Normal")));
}

#[test]
fn cli_mode_command_switch_debug() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":mode debug\n:mode\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Debug"));
}

#[test]
fn cli_clear_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin("1 + 1\n:clear\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared").or(predicate::str::contains("History")));
}

#[test]
fn cli_reset_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin("let x = 10\n:reset\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("reset").or(predicate::str::contains("Bindings")));
}

#[test]
fn cli_history_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin("1 + 1\n:history\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 + 1").or(predicate::str::contains("History")));
}

#[test]
fn cli_inspect_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":inspect 42\n:quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Type").or(predicate::str::contains("Integer")));
}

#[test]
fn cli_ast_command() {
    repl_cmd()
        .arg("repl")
        .write_stdin(":ast 2 + 3\n:quit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Binary")
                .or(predicate::str::contains("Add"))
                .or(predicate::str::contains("Literal")),
        );
}

// Property-style test: Valid integers should always work
#[test]
fn cli_valid_integers_succeed() {
    for n in [-1000, -1, 0, 1, 42, 100, 1000] {
        repl_cmd()
            .arg("repl")
            .write_stdin(format!("{n}\n:quit\n"))
            .assert()
            .success()
            .stdout(predicate::str::contains(n.to_string()));
    }
}

// Property-style test: Valid floats should always work
#[test]
fn cli_valid_floats_succeed() {
    for f in [0.0, 1.5, -2.5, 3.14159] {
        repl_cmd()
            .arg("repl")
            .write_stdin(format!("{f}\n:quit\n"))
            .assert()
            .success();
        // Note: float formatting may vary, so we just check success
    }
}

// Property-style test: Valid strings should always work
#[test]
fn cli_valid_strings_succeed() {
    for s in ["hello", "world", "123", "!@#$%"] {
        repl_cmd()
            .arg("repl")
            .write_stdin(format!("\"{s}\"\n:quit\n"))
            .assert()
            .success()
            .stdout(predicate::str::contains(s));
    }
}

// Batch execution: Multiple operations in sequence
#[test]
fn cli_batch_execution_sequence() {
    repl_cmd()
        .arg("repl")
        .write_stdin(
            r#"
let x = 10
let y = 20
x + y
:quit
"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

// Performance test: Should handle many operations quickly
#[test]
fn cli_batch_many_operations() {
    let mut input = String::new();
    for i in 1..=10 {
        input.push_str(&format!("{i}\n"));
    }
    input.push_str(":quit\n");

    repl_cmd().arg("repl").write_stdin(input).assert().success();
}
