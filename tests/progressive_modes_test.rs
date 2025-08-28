// Test for REPL-UX-002: Progressive modes (standard, test, debug) with contextual features
// Validates comprehensive progressive mode system with mode activation and contextual features

use ruchy::runtime::Repl;

#[test]
fn test_standard_mode_default() {
    let repl = Repl::new().unwrap();
    assert_eq!(repl.get_mode(), "normal");
    assert_eq!(repl.get_prompt(), "ruchy> ");
}

#[test]
fn test_mode_switching_commands() {
    let mut repl = Repl::new().unwrap();
    
    // Test switching to test mode
    let result = repl.eval(":test").unwrap();
    assert!(result.contains("Switched to test mode"));
    assert_eq!(repl.get_mode(), "test");
    assert_eq!(repl.get_prompt(), "test> ");
    
    // Test switching to debug mode
    let result = repl.eval(":debug").unwrap();
    assert!(result.contains("Switched to debug mode"));
    assert_eq!(repl.get_mode(), "debug");
    assert_eq!(repl.get_prompt(), "debug> ");
    
    // Test switching back to normal
    let result = repl.eval(":normal").unwrap();
    assert!(result.contains("Switched to normal mode"));
    assert_eq!(repl.get_mode(), "normal");
    assert_eq!(repl.get_prompt(), "ruchy> ");
}

#[test]
fn test_progressive_mode_activation() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.get_mode(), "normal");
    
    // Test #[test] attribute activation
    let result = repl.eval("#[test]").unwrap();
    assert!(result.contains("Activated test mode"));
    assert_eq!(repl.get_mode(), "test");
    
    // Reset to normal
    let _ = repl.eval(":normal");
    
    // Test #[debug] attribute activation
    let result = repl.eval("#[debug]").unwrap();
    assert!(result.contains("Activated debug mode"));
    assert_eq!(repl.get_mode(), "debug");
}

#[test]
fn test_test_mode_assertions() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":test");
    
    // Test passing assertion
    let result = repl.eval("assert 1 + 1 == 2").unwrap();
    assert!(result.contains("✓ Pass"));
    
    // Test failing assertion
    let result = repl.eval("assert 1 + 1 == 3").unwrap();
    assert!(result.contains("✗ Fail: assertion failed"));
    
    // Test non-boolean assertion
    let result = repl.eval("assert 42").unwrap();
    assert!(result.contains("✗ Fail: assertion must be boolean"));
}

#[test]
fn test_test_mode_regular_evaluation() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":test");
    
    // Regular expressions should get ✓ prefix in test mode
    let result = repl.eval("1 + 1").unwrap();
    assert!(result.starts_with("✓"));
    assert!(result.contains("2"));
    
    let result = repl.eval("\"hello\"").unwrap();
    assert!(result.starts_with("✓"));
    assert!(result.contains("hello"));
}

#[test]
fn test_debug_mode_trace_output() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":debug");
    
    let result = repl.eval("42").unwrap();
    
    // Should contain trace box format
    assert!(result.contains("┌─ Trace ────────┐"));
    assert!(result.contains("│ parse:"));
    assert!(result.contains("│ type:"));
    assert!(result.contains("│ eval:"));
    assert!(result.contains("│ alloc:"));
    assert!(result.contains("└────────────────┘"));
    
    // Should contain type information
    assert!(result.contains("Int"));
    assert!(result.contains("42"));
}

#[test]
fn test_debug_mode_timing_display() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":debug");
    
    let result = repl.eval("1 + 1").unwrap();
    
    // Should show timing in milliseconds
    assert!(result.contains("ms"));
    assert!(result.contains("B"));  // bytes for allocation
    
    // Should show result with type
    assert!(result.contains("Int"));
    assert!(result.contains("2"));
}

#[test]
fn test_mode_persistence_across_evaluations() {
    let mut repl = Repl::new().unwrap();
    
    // Switch to test mode
    let _ = repl.eval(":test");
    assert_eq!(repl.get_mode(), "test");
    
    // Mode should persist across multiple evaluations
    let result1 = repl.eval("1 + 1").unwrap();
    assert!(result1.starts_with("✓"));
    assert_eq!(repl.get_mode(), "test");
    
    let result2 = repl.eval("2 + 2").unwrap();
    assert!(result2.starts_with("✓"));
    assert_eq!(repl.get_mode(), "test");
    
    // Switch to debug mode
    let _ = repl.eval(":debug");
    assert_eq!(repl.get_mode(), "debug");
    
    // Should now show debug traces
    let result3 = repl.eval("3 + 3").unwrap();
    assert!(result3.contains("┌─ Trace ────────┐"));
    assert_eq!(repl.get_mode(), "debug");
}

#[test]
fn test_colon_commands_work_in_all_modes() {
    let mut repl = Repl::new().unwrap();
    
    // Test mode
    let _ = repl.eval(":test");
    let result = repl.eval(":help").unwrap();
    assert!(result.contains("help mode"));
    
    // Debug mode  
    let _ = repl.eval(":debug");
    let result = repl.eval(":modes").unwrap();
    assert!(result.contains("Available modes"));
    
    // Should be able to switch modes from any mode
    let _ = repl.eval(":normal");
    assert_eq!(repl.get_mode(), "normal");
}

#[test]
fn test_type_inference_coverage() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":debug");
    
    // Test various types are correctly inferred
    let int_result = repl.eval("42").unwrap();
    assert!(int_result.contains("Int"));
    
    let float_result = repl.eval("3.14").unwrap();
    assert!(float_result.contains("Float"));
    
    let string_result = repl.eval("\"test\"").unwrap();
    assert!(string_result.contains("String"));
    
    let bool_result = repl.eval("true").unwrap();
    assert!(bool_result.contains("Bool"));
    
    let list_result = repl.eval("[1, 2, 3]").unwrap();
    assert!(list_result.contains("List"));
}

#[test]
fn test_modes_help_command() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval(":modes").unwrap();
    assert!(result.contains("Available modes:"));
    assert!(result.contains("normal - Standard Ruchy evaluation"));
    assert!(result.contains("test   - Assertions and table tests"));
    assert!(result.contains("debug  - Debug information with traces"));
    assert!(result.contains("time   - Execution timing"));
}

#[test]
fn test_exit_from_modes() {
    let mut repl = Repl::new().unwrap();
    
    // Switch to test mode
    let _ = repl.eval(":test");
    assert_eq!(repl.get_mode(), "test");
    
    // :exit should return to normal
    let result = repl.eval(":exit").unwrap();
    assert!(result.contains("Exited to normal mode"));
    assert_eq!(repl.get_mode(), "normal");
    
    // Test from debug mode
    let _ = repl.eval(":debug");
    let result = repl.eval(":exit").unwrap();
    assert!(result.contains("Exited to normal mode"));
    assert_eq!(repl.get_mode(), "normal");
}

#[test]
fn test_table_test_recognition() {
    let mut repl = Repl::new().unwrap();
    let _ = repl.eval(":test");
    
    // Table test should be recognized (even if not fully implemented)
    let result = repl.eval("table_test!((1, 2, 3), (0, 0, 0)) |a, b, c| a + b == c").unwrap();
    assert!(result.contains("✓ Table test recognized"));
}

#[test]
fn test_progressive_activation_only_affects_current_input() {
    let mut repl = Repl::new().unwrap();
    assert_eq!(repl.get_mode(), "normal");
    
    // Progressive activation should activate mode
    let _ = repl.eval("#[test]");
    assert_eq!(repl.get_mode(), "test");
    
    // Subsequent evaluations should remain in that mode
    let result = repl.eval("1 + 1").unwrap();
    assert!(result.starts_with("✓"));
    assert_eq!(repl.get_mode(), "test");
}