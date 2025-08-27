// REPL Magic Commands Test Suite
// Testing magic commands and history mechanism

use ruchy::runtime::repl::Repl;
use std::fs;
use std::io::Write;

// Helper to strip quotes from string output
fn strip_quotes(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

#[test]
fn test_time_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test %time command
    let result = repl.eval("%time 2 + 2").unwrap();
    assert!(result.contains("Executed in:"));
    assert!(result.contains("4")); // Should show the result
}

#[test]
fn test_timeit_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test %timeit command  
    let result = repl.eval("%timeit 2 + 2").unwrap();
    assert!(result.contains("loops"));
    assert!(result.contains("average:"));
    assert!(result.contains("4")); // Should show result
}

#[test]
fn test_run_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    // Create a test script
    let script_path = "/tmp/test_script.ruchy";
    let mut file = fs::File::create(script_path).unwrap();
    writeln!(file, "let x = 10").unwrap();
    writeln!(file, "let y = 20").unwrap();
    writeln!(file, "x + y").unwrap();
    drop(file);
    
    // Test %run command - returns all evaluated expressions
    let result = repl.eval(&format!("%run {}", script_path)).unwrap();
    // %run shows all outputs from the script
    assert!(result.contains("30"));
    
    // Check that variables are available in REPL
    let x_result = repl.eval("x").unwrap();
    assert_eq!(x_result, "10");
    
    let y_result = repl.eval("y").unwrap();
    assert_eq!(y_result, "20");
    
    // Clean up
    let _ = fs::remove_file(script_path);
}

#[test]
fn test_help_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test %help command
    let result = repl.eval("%help").unwrap();
    assert!(result.contains("%time"));
    assert!(result.contains("%timeit"));
    assert!(result.contains("%run"));
    assert!(result.contains("%help"));
}

#[test]
fn test_history_underscore_variable() {
    let mut repl = Repl::new().unwrap();
    
    // Execute some expressions
    repl.eval("42").unwrap();
    
    // Test _ variable (last result)
    let result = repl.eval("_").unwrap();
    assert_eq!(result, "42");
    
    // Execute another expression
    repl.eval("100").unwrap();
    
    // Test _ is updated
    let result = repl.eval("_").unwrap();
    assert_eq!(result, "100");
    
    // Test using _ in expressions
    let result = repl.eval("_ * 2").unwrap();
    assert_eq!(result, "200");
}

#[test]
fn test_history_indexed_variables() {
    let mut repl = Repl::new().unwrap();
    
    // Execute multiple expressions
    repl.eval("10").unwrap();
    repl.eval("20").unwrap();
    repl.eval("30").unwrap();
    
    // Test indexed history variables
    let result = repl.eval("_1").unwrap();
    assert_eq!(result, "10");
    
    let result = repl.eval("_2").unwrap();
    assert_eq!(result, "20");
    
    let result = repl.eval("_3").unwrap();
    assert_eq!(result, "30");
    
    // Test using indexed variables in expressions
    let result = repl.eval("_1 + _2 + _3").unwrap();
    assert_eq!(result, "60");
}

#[test]
fn test_invalid_magic_command() {
    let mut repl = Repl::new().unwrap();
    
    // Test unknown magic command
    let result = repl.eval("%unknown").unwrap();
    assert!(result.contains("Unknown magic command"));
    assert!(result.contains("%help"));
}

#[test]
fn test_magic_command_with_complex_expression() {
    let mut repl = Repl::new().unwrap();
    
    // Test %time with complex expression
    let result = repl.eval("%time [1, 2, 3].map(|x| x * 2)").unwrap();
    assert!(result.contains("Executed in:"));
    assert!(result.contains("[2, 4, 6]"));
}

#[test]
fn test_run_script_with_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Create a script with function definitions
    let script_path = "/tmp/test_functions.ruchy";
    let mut file = fs::File::create(script_path).unwrap();
    writeln!(file, "fn add(a, b) {{ a + b }}").unwrap();
    writeln!(file, "fn multiply(a, b) {{ a * b }}").unwrap();
    writeln!(file, "add(10, 20)").unwrap();
    drop(file);
    
    // Run the script  
    let result = repl.eval(&format!("%run {}", script_path)).unwrap();
    assert!(result.contains("30"));
    
    // Test that functions are available
    let result = repl.eval("multiply(5, 6)").unwrap();
    assert_eq!(result, "30");
    
    // Clean up
    let _ = fs::remove_file(script_path);
}

#[test]
fn test_history_persistence_across_expressions() {
    let mut repl = Repl::new().unwrap();
    
    // Build up history
    repl.eval("\"first\"").unwrap();
    repl.eval("\"second\"").unwrap();
    repl.eval("\"third\"").unwrap();
    
    // Test that all history is accessible
    // Note: each eval adds to history, so indices will shift
    assert_eq!(strip_quotes(&repl.eval("_1").unwrap()), "first");
    assert_eq!(strip_quotes(&repl.eval("_2").unwrap()), "second");  
    assert_eq!(strip_quotes(&repl.eval("_3").unwrap()), "third");
    
    // At this point history is: ["first", "second", "third", "first", "second", "third"]
    // So _ is "third" (the last eval result)
    assert_eq!(strip_quotes(&repl.eval("_").unwrap()), "third");
    
    // Add one more - it becomes _8 because we've done 7 evals so far
    repl.eval("\"fourth\"").unwrap();
    
    // Now history has 8 items, _8 is "fourth", and _ is "fourth"
    assert_eq!(strip_quotes(&repl.eval("_").unwrap()), "fourth");
}

#[test]
fn test_timeit_with_iterations() {
    let mut repl = Repl::new().unwrap();
    
    // Test that timeit runs multiple iterations
    let result = repl.eval("%timeit 1 + 1").unwrap();
    assert!(result.contains("1000 loops"));
    assert!(result.contains("2")); // Result should still be shown
}

#[test]
fn test_run_nonexistent_file() {
    let mut repl = Repl::new().unwrap();
    
    // Test error handling for non-existent file
    let result = repl.eval("%run /nonexistent/file.ruchy");
    // Should return an error message about the file not existing
    assert!(result.is_err() || result.unwrap().contains("Failed"));
}