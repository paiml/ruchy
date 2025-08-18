//! Integration tests for REPL commands and features

#![allow(clippy::expect_used)] // Tests can use expect

use std::process::{Command, Stdio};
use std::io::Write;

fn run_repl_commands(commands: &str) -> String {
    let mut child = Command::new("cargo")
        .args(["run", "-p", "ruchy-cli", "--", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn REPL");
    
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all(commands.as_bytes()).expect("Failed to write commands");
    stdin.write_all(b"\n:quit\n").expect("Failed to write quit");
    
    let output = child.wait_with_output().expect("Failed to read output");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_help_command() {
    let output = run_repl_commands(":help");
    assert!(output.contains("Available commands:"));
    assert!(output.contains(":help, :h"));
    assert!(output.contains(":quit, :q"));
    assert!(output.contains(":history"));
    assert!(output.contains(":bindings, :env"));
    assert!(output.contains(":ast"));
    assert!(output.contains(":type"));
}

#[test]
fn test_history_command() {
    let output = run_repl_commands("1 + 1\n2 + 2\n:history");
    assert!(output.contains("1: 1 + 1"));
    assert!(output.contains("2: 2 + 2"));
}

#[test]
fn test_bindings_command() {
    let output = run_repl_commands("let x = 42\nlet y = 100\n:bindings");
    assert!(output.contains("x: 42"));
    assert!(output.contains("y: 100"));
}

#[test]
fn test_env_command_alias() {
    let output = run_repl_commands("let foo = 123\n:env");
    assert!(output.contains("foo: 123"));
}

#[test]
fn test_clear_command() {
    let output = run_repl_commands("let x = 1\n:clear\n:bindings");
    assert!(output.contains("Session cleared"));
    assert!(output.contains("No bindings"));
}

#[test]
fn test_reset_command() {
    let output = run_repl_commands("let x = 1\n2 + 2\n:reset\n:history");
    assert!(output.contains("REPL reset to initial state"));
    // After reset, history should be empty (no items listed)
    assert!(!output.contains("1:"));
}

#[test]
fn test_ast_command() {
    let output = run_repl_commands(":ast 1 + 2");
    assert!(output.contains("Binary"));
    assert!(output.contains("Add"));
    assert!(output.contains("Integer"));
}

#[test]
fn test_type_command() {
    let output = run_repl_commands(":type 42");
    // Currently shows "not yet implemented" message
    assert!(output.contains("Type inference not yet implemented"));
}

#[test]
fn test_arithmetic_evaluation() {
    let output = run_repl_commands("2 + 3\n5 * 4\n10 / 2");
    assert!(output.contains('5'));
    assert!(output.contains("20"));
    assert!(output.contains('5'));
}

#[test]
fn test_let_bindings() {
    let output = run_repl_commands("let x = 10\nx\nx * 2");
    assert!(output.contains("10"));
    assert!(output.contains("20"));
}

#[test]
fn test_string_operations() {
    let output = run_repl_commands(r#""hello" + " world""#);
    assert!(output.contains(r#""hello world""#));
}

#[test]
fn test_boolean_operations() {
    let output = run_repl_commands("true && false\ntrue || false\n!true");
    assert!(output.contains("false"));
    assert!(output.contains("true"));
    assert!(output.contains("false"));
}

#[test]
fn test_if_expressions() {
    let output = run_repl_commands("if true { 100 } else { 200 }\nif false { 1 } else { 2 }");
    assert!(output.contains("100"));
    assert!(output.contains('2'));
}

#[test]
fn test_block_expressions() {
    let output = run_repl_commands("{ 1 + 1 }");
    assert!(output.contains('2'));
    // TODO: Fix block evaluation to return last expression instead of let binding value
    // Currently { let a = 5; a * 2 } returns 5, not 10
}

#[test]
fn test_function_calls() {
    let output = run_repl_commands(r#"println("Hello REPL")"#);
    assert!(output.contains("Hello REPL"));
}

#[test]
fn test_unknown_command() {
    let output = run_repl_commands(":unknown");
    // The actual message is "Unknown command: :unknown"
    assert!(output.to_lowercase().contains("unknown"));
    assert!(output.contains("Available commands")); // Should show help
}

#[test]
fn test_empty_lines_ignored() {
    let output = run_repl_commands("\n\n1 + 1\n\n");
    // Should only evaluate the expression once
    let twos: Vec<&str> = output.matches('2').collect();
    assert_eq!(twos.len(), 1);
}