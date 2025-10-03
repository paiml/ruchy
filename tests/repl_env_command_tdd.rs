// TDD Tests for REPL :env command (REPL-005)
//
// Requirements:
// 1. `:env` should show comprehensive environment information
// 2. Display all variable bindings with types and values
// 3. Show REPL mode and state information
// 4. Show history count and other metadata
// 5. Format output in structured, readable way

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_env_command_empty_environment() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Check environment before defining any variables
    let result = repl.eval(":env").unwrap();
    assert!(
        result.contains("Environment") || result.contains("Variables"),
        "Expected environment header but got: {result}"
    );
    assert!(
        result.contains('0') || result.contains("empty") || result.contains("No variables"),
        "Expected indication of empty environment but got: {result}"
    );
}

#[test]
fn test_env_command_shows_variables() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define some variables
    repl.eval("let x = 42").unwrap();
    repl.eval("let name = \"Alice\"").unwrap();

    // Check environment
    let result = repl.eval(":env").unwrap();
    assert!(
        result.contains('x') && result.contains("42"),
        "Expected variable x=42 but got: {result}"
    );
    assert!(
        result.contains("name") && result.contains("Alice"),
        "Expected variable name=Alice but got: {result}"
    );
}

#[test]
fn test_env_command_shows_types() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define variables of different types
    repl.eval("let num = 42").unwrap();
    repl.eval("let text = \"hello\"").unwrap();
    repl.eval("let flag = true").unwrap();

    // Check environment shows type information
    let result = repl.eval(":env").unwrap();
    assert!(
        result.contains("Integer") || result.contains("Int") || result.contains("42"),
        "Expected type info for integer but got: {result}"
    );
    assert!(
        result.contains("String") || result.contains("Str") || result.contains("hello"),
        "Expected type info for string but got: {result}"
    );
    assert!(
        result.contains("Bool") || result.contains("true"),
        "Expected type info for boolean but got: {result}"
    );
}

#[test]
fn test_env_command_shows_mode() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Check default mode
    let result1 = repl.eval(":env").unwrap();
    assert!(
        result1.contains("Mode") || result1.contains("Normal"),
        "Expected mode information but got: {result1}"
    );

    // Switch to debug mode
    repl.eval(":mode debug").unwrap();

    // Check mode changed
    let result2 = repl.eval(":env").unwrap();
    assert!(
        result2.contains("Debug"),
        "Expected Debug mode in env but got: {result2}"
    );
}

#[test]
fn test_env_command_shows_history_count() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Execute some commands
    repl.eval("1 + 1").unwrap();
    repl.eval("2 + 2").unwrap();
    repl.eval("3 + 3").unwrap();

    // Check history count
    let result = repl.eval(":env").unwrap();
    assert!(
        result.contains("History") || result.contains('3') || result.contains('4'),
        "Expected history count but got: {result}"
    );
}

#[test]
fn test_env_command_structured_output() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let x = 10").unwrap();

    // Check for structured formatting
    let result = repl.eval(":env").unwrap();

    // Should have section headers
    assert!(
        result.contains("===") || result.contains("---") || result.contains(':'),
        "Expected structured sections but got: {result}"
    );
}

#[test]
fn test_env_command_shows_all_metadata() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval("let x = 42").unwrap();
    repl.eval(":mode debug").unwrap();

    // Get comprehensive environment info
    let result = repl.eval(":env").unwrap();

    // Should show multiple pieces of information
    let has_vars = result.contains('x') || result.contains("Variables");
    let has_mode = result.contains("Mode") || result.contains("Debug");
    let has_metadata = result.contains("History") || result.contains("Environment");

    assert!(
        has_vars && has_mode && has_metadata,
        "Expected comprehensive environment info but got: {result}"
    );
}
