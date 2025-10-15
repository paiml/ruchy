// CLI Contract Tests for `ruchy actor:observe` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_actor_observe_help() {
    ruchy_cmd()
        .arg("actor:observe")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Actor").or(predicate::str::contains("observatory")));
}

#[test]
#[ignore = "Actor observatory is long-running server"]
fn test_actor_observe_starts() {
    ruchy_cmd()
        .arg("actor:observe")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .code(predicate::ne(2));
}
