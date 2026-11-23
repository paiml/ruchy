#![allow(missing_docs)]
// CLI Contract Tests for `ruchy mcp` command
use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_mcp_help() {
    ruchy_cmd()
        .arg("mcp")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP").or(predicate::str::contains("server")));
}

#[test]
fn test_mcp_starts() {
    ruchy_cmd()
        .arg("mcp")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .code(predicate::ne(2));
}
