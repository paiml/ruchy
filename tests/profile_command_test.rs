// Test for REPL-MAGIC-002: %profile flamegraph generation
// Tests the %profile command for performance analysis

use ruchy::runtime::Repl;

#[test]
fn test_profile_command_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Profile a simple expression
    let profile_output = repl.eval("%profile 2 + 2").unwrap();
    
    // Check that profile output contains expected sections
    assert!(profile_output.contains("Performance Profile"), "Should show profile header");
    assert!(profile_output.contains("Expression: 2 + 2"), "Should show profiled expression");
    assert!(profile_output.contains("Result: 4"), "Should show result");
    assert!(profile_output.contains("Timing Breakdown"), "Should show timing section");
    assert!(profile_output.contains("Parse:"), "Should show parse time");
    assert!(profile_output.contains("Evaluate:"), "Should show eval time");
    assert!(profile_output.contains("Total:"), "Should show total time");
    assert!(profile_output.contains("Memory Usage"), "Should show memory section");
    assert!(profile_output.contains("AST size:"), "Should show AST size");
    assert!(profile_output.contains("Analysis"), "Should show analysis section");
}

#[test]
fn test_profile_command_no_args() {
    let mut repl = Repl::new().unwrap();
    
    // Test with no arguments
    let result = repl.eval("%profile").unwrap();
    
    assert!(result.contains("Usage: %profile <expression>"), 
            "Should show usage when no arguments provided");
}

#[test]
fn test_profile_command_parse_error() {
    let mut repl = Repl::new().unwrap();
    
    // Profile an invalid expression
    let profile_output = repl.eval("%profile let x =").unwrap();
    
    assert!(profile_output.contains("Parse error:"), 
            "Should show parse error for invalid expression");
}

#[test]
fn test_profile_command_runtime_error() {
    let mut repl = Repl::new().unwrap();
    
    // Profile an expression that causes runtime error
    let profile_output = repl.eval("%profile undefined_var + 1").unwrap();
    
    assert!(profile_output.contains("Evaluation error:"), 
            "Should show evaluation error for undefined variable");
}

#[test]
fn test_profile_command_performance_analysis() {
    let mut repl = Repl::new().unwrap();
    
    // Profile a simple expression that should be fast
    let profile_output = repl.eval("%profile 1 + 1").unwrap();
    
    // Should be marked as fast
    assert!(profile_output.contains("🚀 Fast execution"), 
            "Simple expression should be marked as fast");
}

#[test]
fn test_profile_command_complex_expression() {
    let mut repl = Repl::new().unwrap();
    
    // Profile a more complex expression
    let profile_output = repl.eval("%profile [1, 2, 3, 4, 5].map(|x| x * 2)").unwrap();
    
    // Should contain all the expected sections
    assert!(profile_output.contains("Expression: [1, 2, 3, 4, 5].map(|x| x * 2)"), 
            "Should show complex expression");
    assert!(profile_output.contains("Result: [2, 4, 6, 8, 10]"), 
            "Should show correct result");
    assert!(profile_output.contains("Parse:"), "Should show parse timing");
    assert!(profile_output.contains("Evaluate:"), "Should show eval timing");
}

#[test]
fn test_profile_command_memory_tracking() {
    let mut repl = Repl::new().unwrap();
    
    // Profile an expression that allocates memory
    let profile_output = repl.eval("%profile let large_list = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").unwrap();
    
    // Should show memory usage
    assert!(profile_output.contains("Memory Usage"), "Should show memory section");
    assert!(profile_output.contains("AST size:"), "Should show AST size");
    assert!(profile_output.contains("bytes"), "Should show byte counts");
}

#[test]
fn test_profile_command_percentage_breakdown() {
    let mut repl = Repl::new().unwrap();
    
    // Profile an expression
    let profile_output = repl.eval("%profile 42 * 7").unwrap();
    
    // Should show percentages in timing breakdown
    let timing_section = profile_output.lines()
        .skip_while(|line| !line.contains("Timing Breakdown"))
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    
    // Should have percentage indicators
    assert!(timing_section.contains("%"), "Should show percentages in timing breakdown");
    
    // Parse and eval percentages should roughly add up to 100%
    // (allowing for small rounding errors)
    let parse_line = profile_output.lines()
        .find(|line| line.contains("Parse:"))
        .expect("Should have parse line");
    let eval_line = profile_output.lines()
        .find(|line| line.contains("Evaluate:"))
        .expect("Should have evaluate line");
    
    assert!(parse_line.contains("(") && parse_line.contains("%)"), 
            "Parse line should show percentage");
    assert!(eval_line.contains("(") && eval_line.contains("%)"), 
            "Evaluate line should show percentage");
}