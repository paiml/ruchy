// Test for REPL-MAGIC-001: %debug post-mortem debugging
// Tests the %debug command for analyzing errors

use ruchy::runtime::Repl;

#[test]
fn test_debug_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // First, cause an error
    let result = repl.eval("undefined_var + 1");
    assert!(result.is_err(), "Should fail with undefined variable");
    
    // Now use %debug to inspect the error
    let debug_output = repl.eval("%debug").unwrap();
    
    // Check that debug output contains expected information
    assert!(debug_output.contains("Debug Information"), "Should show debug header");
    assert!(debug_output.contains("Expression: undefined_var + 1"), "Should show failing expression");
    assert!(debug_output.contains("Error: Undefined variable: undefined_var"), "Should show error message");
    assert!(debug_output.contains("Variable Bindings at Error"), "Should show bindings section");
    assert!(debug_output.contains("Stack Trace"), "Should show stack trace section");
}

#[test]
fn test_debug_command_with_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Create some variables
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();
    
    // Cause an error
    let result = repl.eval("x + z");  // z is undefined
    assert!(result.is_err(), "Should fail with undefined variable");
    
    // Check debug output includes bindings
    let debug_output = repl.eval("%debug").unwrap();
    
    assert!(debug_output.contains("x: 10"), "Should show x binding");
    assert!(debug_output.contains("y: 20"), "Should show y binding");
    assert!(debug_output.contains("z"), "Error should mention z");
}

#[test]
fn test_debug_command_no_error() {
    let mut repl = Repl::new().unwrap();
    
    // No error has occurred yet
    let debug_output = repl.eval("%debug").unwrap();
    
    assert!(debug_output.contains("No debug information available"), 
            "Should indicate no debug info when no error occurred");
}

#[test]
fn test_debug_command_clears_on_success() {
    let mut repl = Repl::new().unwrap();
    
    // Cause an error
    let result = repl.eval("bad_var");
    assert!(result.is_err(), "Should fail");
    
    // Debug should work
    let debug_output = repl.eval("%debug").unwrap();
    assert!(debug_output.contains("Debug Information"), "Should have debug info");
    
    // Now have a successful evaluation
    repl.eval("2 + 2").unwrap();
    
    // Debug info should be cleared
    let debug_output = repl.eval("%debug").unwrap();
    assert!(debug_output.contains("No debug information available"), 
            "Should clear debug info after successful evaluation");
}

#[test]
fn test_debug_command_error_chain() {
    let mut repl = Repl::new().unwrap();
    
    // Create an error that might have a chain
    let result = repl.eval("1 / 0");  // Division by zero
    // Note: This might not actually create a chained error in our implementation,
    // but we test that the debug system handles it gracefully
    
    if result.is_err() {
        let debug_output = repl.eval("%debug").unwrap();
        assert!(debug_output.contains("Stack Trace"), "Should show stack trace even for simple errors");
    }
}

#[test]
fn test_debug_command_with_history() {
    let mut repl = Repl::new().unwrap();
    
    // Execute some successful commands to build history
    repl.eval("let a = 1").unwrap();
    repl.eval("let b = 2").unwrap();
    repl.eval("a + b").unwrap();
    
    // Now cause an error
    let result = repl.eval("a + missing_var");
    assert!(result.is_err(), "Should fail with undefined variable");
    
    // Check that debug output includes context from history
    let debug_output = repl.eval("%debug").unwrap();
    
    // Should contain the last successful expression in stack trace
    assert!(debug_output.contains("Last successful expression"), 
            "Should show last successful expression in stack trace");
}