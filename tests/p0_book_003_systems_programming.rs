/// P0-BOOK-003: Systems Programming TDD Tests
/// These tests define the expected behavior for systems programming features
/// Based on ruchy-book chapter 8 examples that are currently broken

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_signal_handler_syntax() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test signal handler syntax (currently broken with "Expected identifier after '::'")
    let code = r#"
import std::signal
signal::on(SIGINT, || {
    println("Graceful shutdown...")
    exit(0)
})
println("Signal handler registered")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Signal handler registered"));
}

#[test]
fn test_object_literal_parsing() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test object literal with string keys (currently broken)
    let code = r#"
let service_config = {
    name: "web_server", 
    command: "./server", 
    port: 8080
}
println("Service: " + service_config.name)
println("Port: " + service_config.port.to_s())
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Service: web_server"))
        .stdout(predicate::str::contains("Port: 8080"));
}

#[test]
fn test_question_mark_operator() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test question mark operator syntax (currently broken with "Unexpected token: Question")
    let code = r#"
import std::fs
let result = read_file("nonexistent.txt")?
println("Should not reach here")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    // For now, test that it parses without panic (may fail at runtime)
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // Success or controlled failure, but not panic
}

#[test]
fn test_for_in_loop_parsing() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test for-in loop syntax (currently broken with "Expected In, found Comma")
    let code = r#"
let items = ["apple", "banana", "cherry"]
for item in items {
    println("Item: " + item)
}
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Item: apple"))
        .stdout(predicate::str::contains("Item: banana"))
        .stdout(predicate::str::contains("Item: cherry"));
}

#[test]
fn test_function_parameter_parsing() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test function with parameters (currently broken with "Expected RightParen, found Colon")
    let code = r#"
fn format_size(bytes: i64) {
    if bytes < 1024 {
        return bytes.to_s() + " B"
    } else if bytes < 1024 * 1024 {
        return (bytes / 1024).to_s() + " KB"
    } else {
        return (bytes / (1024 * 1024)).to_s() + " MB"
    }
}

let size = format_size(2048)
println("Size: " + size)
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Size: 2 KB"));
}

#[test]
fn test_systems_module_imports() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test that std::system and std::process modules can be imported
    let code = r#"
import std::system
import std::process
println("System modules imported successfully")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("System modules imported successfully"));
}

#[test]
fn test_basic_system_functions() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Test basic system function calls (these should work or fail gracefully)
    let code = r#"
import std::system
import std::process

// Basic system info that should be available
let pid = process::current_pid()
println("Current PID: " + pid.to_s())

// These may not be implemented yet, but should not cause parser errors
println("System functions accessible")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("System functions accessible"));
}

#[test]
fn test_system_parsing_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Comprehensive test of system programming syntax - should not panic the parser
    let code = r#"
import std::system
import std::process
import std::signal

fn main() {
    // Object literals
    let config = {name: "test", value: 42}
    
    // For-in loops  
    let items = [1, 2, 3]
    for item in items {
        println(item.to_s())
    }
    
    // Function with typed parameters
    fn helper(x: i32) {
        return x * 2
    }
    
    println("All syntax parsed successfully")
}
"#;
    
    fs::write(&file_path, code).unwrap();
    
    // Should not panic, even if compilation fails
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(&["run", file_path.to_str().unwrap()])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // Success or failure, but not panic
}