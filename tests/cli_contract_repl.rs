// CLI Contract Tests for `ruchy repl` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_repl_help() {
    ruchy_cmd()
        .arg("repl")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("REPL").or(predicate::str::contains("interactive")));
}

#[test]
#[ignore = "REPL is interactive, requires stdin interaction"]
fn test_repl_starts() {
    // This would require rexpect for interactive testing
    // Ignored for now since REPL requires stdin interaction
    ruchy_cmd()
        .arg("repl")
        .write_stdin("exit\n")
        .assert()
        .code(predicate::ne(2));
}
