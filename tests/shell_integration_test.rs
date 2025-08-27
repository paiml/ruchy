// TDD Test Suite for Shell Integration
// Testing shell commands (!command) and shell substitution

use ruchy::runtime::repl::Repl;
use std::fs;
use std::io::Write;

#[test]
fn test_shell_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Test basic shell command
    let result = repl.eval("!echo hello world").unwrap();
    assert!(result.contains("hello world"),
        "Shell command should execute echo, got: {}", result);
}

#[test]
fn test_shell_command_pwd() {
    let mut repl = Repl::new().unwrap();
    
    // Test pwd command
    let result = repl.eval("!pwd").unwrap();
    assert!(result.contains("/"),
        "pwd should return a path, got: {}", result);
}

#[test]
fn test_shell_command_ls() {
    let mut repl = Repl::new().unwrap();
    
    // Create a test file
    let test_file = "/tmp/test_shell_ls.txt";
    fs::write(test_file, "test content").unwrap();
    
    // Test ls command in /tmp
    let result = repl.eval("!ls /tmp | grep test_shell_ls").unwrap();
    assert!(result.contains("test_shell_ls"),
        "ls should show our test file, got: {}", result);
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_shell_substitution() {
    let mut repl = Repl::new().unwrap();
    
    // Test shell substitution in let binding
    let result = repl.eval("let current_dir = !pwd").unwrap();
    assert_eq!(result, "", "Let binding should return empty");
    
    // Check that the variable was set
    let dir_val = repl.eval("current_dir").unwrap();
    assert!(dir_val.contains("/"),
        "current_dir should contain a path, got: {}", dir_val);
}

#[test]
fn test_shell_command_with_args() {
    let mut repl = Repl::new().unwrap();
    
    // Test shell command with arguments
    let result = repl.eval("!echo -n 'no newline'").unwrap();
    assert_eq!(result.trim(), "no newline",
        "Should execute echo without newline");
}

#[test]
fn test_shell_command_piping() {
    let mut repl = Repl::new().unwrap();
    
    // Test shell command with pipes
    let result = repl.eval("!echo 'line1\nline2\nline3' | head -n 1").unwrap();
    assert!(result.contains("line1") && !result.contains("line2"),
        "Pipe should work, got: {}", result);
}

#[test]
fn test_shell_command_error_handling() {
    let mut repl = Repl::new().unwrap();
    
    // Test non-existent command
    let result = repl.eval("!nonexistentcommand123");
    assert!(result.as_ref().is_err() || 
            result.as_ref().unwrap().contains("not found") || 
            result.as_ref().unwrap().contains("Error"),
        "Should handle non-existent command gracefully");
}

#[test]
fn test_shell_command_in_expression() {
    let mut repl = Repl::new().unwrap();
    
    // Test using shell command result in expression
    let result = repl.eval("let files = !ls /tmp").unwrap();
    assert_eq!(result, "", "Let binding returns empty");
    
    // Check that files contains something
    let files_val = repl.eval("files").unwrap();
    assert!(!files_val.is_empty(),
        "files should contain ls output");
}

#[test]
fn test_shell_command_multiline_output() {
    let mut repl = Repl::new().unwrap();
    
    // Test command with multiline output
    let result = repl.eval("!echo -e 'line1\\nline2\\nline3'").unwrap();
    assert!(result.contains("line1") && result.contains("line2") && result.contains("line3"),
        "Should preserve multiline output, got: {}", result);
}

#[test]
fn test_shell_command_environment() {
    let mut repl = Repl::new().unwrap();
    
    // Test accessing environment variables
    let result = repl.eval("!echo $HOME").unwrap();
    assert!(result.contains("/"),
        "Should expand $HOME variable, got: {}", result);
}

#[test]
fn test_shell_command_cd() {
    let mut repl = Repl::new().unwrap();
    
    // Note: cd doesn't persist across shell invocations
    // This is expected behavior - each ! command runs in a new shell
    // Test that we can run cd and pwd in the same command
    let result = repl.eval("!cd /tmp && pwd").unwrap();
    assert!(result.contains("/tmp"),
        "Combined cd && pwd should show /tmp, got: {}", result);
    
    // Verify that directory change doesn't persist
    let pwd = repl.eval("!pwd").unwrap();
    assert!(!pwd.contains("/tmp"),
        "Directory change should not persist, got: {}", pwd);
}

#[test]
fn test_shell_command_output_capture() {
    let mut repl = Repl::new().unwrap();
    
    // Test capturing command output as string
    repl.eval("let count = !ls /tmp | wc -l").unwrap();
    let count_val = repl.eval("count").unwrap();
    
    // Should be a number (as string, potentially in quotes)
    let cleaned = count_val.trim().trim_matches('"');
    assert!(cleaned.chars().all(|c| c.is_ascii_digit()),
        "Should capture numeric output, got: {}", count_val);
}

#[test]
fn test_shell_command_stderr() {
    let mut repl = Repl::new().unwrap();
    
    // Test command that writes to stderr
    let result = repl.eval("!ls /nonexistent 2>&1");
    assert!(result.as_ref().is_err() || 
            result.as_ref().unwrap().contains("No such") || 
            result.as_ref().unwrap().contains("cannot access"),
        "Should capture stderr output");
}

#[test]
fn test_shell_escape_prevention() {
    let mut repl = Repl::new().unwrap();
    
    // Test that shell commands in strings don't execute
    let result = repl.eval("\"!echo should not execute\"").unwrap();
    assert_eq!(result, "\"!echo should not execute\"",
        "String should not execute shell command");
    
    // Test that only lines starting with ! are shell commands
    let result = repl.eval("let x = 5 !echo bad").unwrap_or_else(|e| e.to_string());
    assert!(!result.contains("bad"),
        "Should not execute inline shell command");
}