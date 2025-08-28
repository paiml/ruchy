// TDD Test Suite for Workspace Commands
// Testing whos(), clear!(), save_image() and other workspace management

use ruchy::runtime::repl::Repl;
use std::fs;

#[test]
fn test_whos_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Define some variables
    repl.eval("let x = 42").unwrap();
    repl.eval("let name = \"Alice\"").unwrap();
    repl.eval("fn greet() { \"Hello\" }").unwrap();
    
    // Test whos() command
    let result = repl.eval("whos()").unwrap();
    assert!(result.contains("x") && result.contains("Integer"),
        "whos() should list x with type, got: {}", result);
    assert!(result.contains("name") && result.contains("String"),
        "whos() should list name with type, got: {}", result);
    assert!(result.contains("greet") && result.contains("Function"),
        "whos() should list greet function, got: {}", result);
}

#[test]
fn test_whos_with_filter() {
    let mut repl = Repl::new().unwrap();
    
    // Define various types
    repl.eval("let num1 = 10").unwrap();
    repl.eval("let num2 = 20").unwrap();
    repl.eval("let str1 = \"hello\"").unwrap();
    repl.eval("let list1 = [1, 2, 3]").unwrap();
    
    // Test whos() with type filter
    let result = repl.eval("whos(\"Integer\")").unwrap();
    assert!(result.contains("num1") && result.contains("num2"),
        "whos(\"Integer\") should show only integers, got: {}", result);
    assert!(!result.contains("str1"),
        "whos(\"Integer\") should not show strings, got: {}", result);
}

#[test]
fn test_clear_bang_command() {
    let mut repl = Repl::new().unwrap();
    
    // Define some variables
    repl.eval("let x = 100").unwrap();
    repl.eval("let y = 200").unwrap();
    
    // Verify they exist
    assert_eq!(repl.eval("x").unwrap(), "100");
    assert_eq!(repl.eval("y").unwrap(), "200");
    
    // Test clear_all() command - clears all user-defined variables
    let result = repl.eval("clear_all()").unwrap();
    assert!(result.contains("cleared") || result.contains("Cleared") || result.is_empty(),
        "clear_all() should indicate workspace cleared, got: {}", result);
    
    // Variables should be gone
    let x_result = repl.eval("x");
    assert!(x_result.is_err() || x_result.unwrap().contains("undefined"),
        "x should be undefined after clear!()");
}

#[test]
fn test_clear_bang_selective() {
    let mut repl = Repl::new().unwrap();
    
    // Define variables
    repl.eval("let keep_me = 42").unwrap();
    repl.eval("let delete_me = 99").unwrap();
    repl.eval("let delete_also = \"bye\"").unwrap();
    
    // Test selective clear with pattern
    let result = repl.eval("clear_all(\"delete*\")").unwrap();
    assert!(result.contains("2") || result.contains("cleared"),
        "clear_all(pattern) should clear matching vars, got: {}", result);
    
    // Check what remains
    assert_eq!(repl.eval("keep_me").unwrap(), "42", "keep_me should remain");
    let delete_result = repl.eval("delete_me");
    assert!(delete_result.is_err() || 
            delete_result.unwrap().contains("undefined"),
        "delete_me should be cleared");
}

#[test]
fn test_save_image_command() {
    let mut repl = Repl::new().unwrap();
    let image_file = "/tmp/test_workspace.ruchy";
    
    // Create a workspace
    repl.eval("let pi = 3.14159").unwrap();
    repl.eval("let data = [1, 2, 3, 4, 5]").unwrap();
    repl.eval("fn square(x) { x * x }").unwrap();
    
    // Save workspace image
    let result = repl.eval(&format!("save_image(\"{}\")", image_file)).unwrap();
    assert!(!result.contains("Error") && !result.contains("failed"),
        "save_image() should succeed, got: {}", result);
    
    // Check file exists
    assert!(fs::metadata(image_file).is_ok(),
        "Image file should be created");
    
    // Load in new REPL
    let mut new_repl = Repl::new().unwrap();
    new_repl.eval(&format!(":load {}", image_file)).unwrap();
    
    // Verify workspace restored
    assert_eq!(new_repl.eval("pi").unwrap(), "3.14159");
    assert!(new_repl.eval("data").unwrap().contains("1") && 
            new_repl.eval("data").unwrap().contains("5"));
    assert_eq!(new_repl.eval("square(3)").unwrap(), "9");
    
    // Clean up
    let _ = fs::remove_file(image_file);
}

#[test]
fn test_who_command() {
    let mut repl = Repl::new().unwrap();
    
    // who() is like whos() but simpler output
    repl.eval("let a = 1").unwrap();
    repl.eval("let b = 2").unwrap();
    repl.eval("let c = 3").unwrap();
    
    let result = repl.eval("who()").unwrap();
    assert!(result.contains("a") && result.contains("b") && result.contains("c"),
        "who() should list all variables, got: {}", result);
}

#[test]
fn test_workspace_command() {
    let mut repl = Repl::new().unwrap();
    
    // workspace() shows current workspace info
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();
    repl.eval("fn test() { 42 }").unwrap();
    
    let result = repl.eval("workspace()").unwrap();
    assert!(result.contains("3") || result.contains("variables") || 
            result.contains("bindings"),
        "workspace() should show count, got: {}", result);
}

#[test]
fn test_locals_command() {
    let mut repl = Repl::new().unwrap();
    
    // locals() shows local scope variables
    repl.eval("let global = 100").unwrap();
    repl.eval("fn test() { let local = 50; locals() }").unwrap();
    
    // In global scope
    let result = repl.eval("locals()").unwrap();
    assert!(result.contains("global"),
        "locals() should show global var, got: {}", result);
}

#[test]
fn test_globals_command() {
    let mut repl = Repl::new().unwrap();
    
    // globals() shows all global variables
    repl.eval("let global1 = 1").unwrap();
    repl.eval("let global2 = 2").unwrap();
    
    let result = repl.eval("globals()").unwrap();
    assert!(result.contains("global1") && result.contains("global2"),
        "globals() should show all globals, got: {}", result);
}

#[test]
fn test_reset_command() {
    let mut repl = Repl::new().unwrap();
    
    // Add state
    repl.eval("let x = 42").unwrap();
    repl.eval("fn test() { x }").unwrap();
    
    // reset() clears everything and restarts
    let result = repl.eval("reset()").unwrap();
    assert!(result.contains("reset") || result.contains("Reset") || 
            result.contains("cleared"),
        "reset() should indicate reset, got: {}", result);
    
    // Everything should be gone
    assert!(repl.eval("x").is_err() || 
            repl.eval("x").unwrap().contains("undefined"));
}

#[test]
fn test_del_command() {
    let mut repl = Repl::new().unwrap();
    
    // Create variables
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();
    
    // Delete specific variable
    let result = repl.eval("del(x)").unwrap();
    assert!(!result.contains("Error"),
        "del(x) should succeed, got: {}", result);
    
    // x should be gone, y should remain
    assert!(repl.eval("x").is_err() || 
            repl.eval("x").unwrap().contains("undefined"));
    assert_eq!(repl.eval("y").unwrap(), "20");
}

#[test]
fn test_exists_command() {
    let mut repl = Repl::new().unwrap();
    
    // Define a variable
    repl.eval("let defined = 42").unwrap();
    
    // Test exists() function
    let result = repl.eval("exists(\"defined\")").unwrap();
    assert!(result.contains("true") || result.contains("True"),
        "exists(\"defined\") should be true, got: {}", result);
    
    let result = repl.eval("exists(\"undefined\")").unwrap();
    assert!(result.contains("false") || result.contains("False"),
        "exists(\"undefined\") should be false, got: {}", result);
}

#[test]
fn test_memory_info() {
    let mut repl = Repl::new().unwrap();
    
    // Create some data
    repl.eval("let big_list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").unwrap();
    
    // Test memory_info() command
    let result = repl.eval("memory_info()").unwrap();
    assert!(result.contains("memory") || result.contains("Memory") ||
            result.contains("bytes") || result.contains("KB"),
        "memory_info() should show memory usage, got: {}", result);
}

#[test]
fn test_time_info() {
    let mut repl = Repl::new().unwrap();
    
    // Test time_info() - shows session time
    let result = repl.eval("time_info()").unwrap();
    assert!(result.contains("session") || result.contains("Session") ||
            result.contains("time") || result.contains("seconds"),
        "time_info() should show session info, got: {}", result);
}