// TDD Test Suite for Tab Completion Rustyline Integration
// Testing that the rustyline Completer trait is properly implemented

use ruchy::runtime::completion::RuchyCompleter;
use rustyline::completion::Completer;
use rustyline::history::DefaultHistory;
use rustyline::Context;

#[test]
fn test_completer_trait_basic_functionality() {
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // Test basic method completion
    let input = "\"hello\".";
    let pos = input.len();
    
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Completer should not fail on basic input");
    
    let (start, pairs) = result.unwrap();
    assert!(start <= pos, "Start position should be valid");
    assert!(!pairs.is_empty(), "Should return some completions for string methods");
    
    // Check that we get string method completions
    let completions: Vec<String> = pairs.into_iter().map(|p| p.replacement).collect();
    assert!(
        completions.iter().any(|s| s.contains("len") || s.contains("split") || s.contains("trim")),
        "Should suggest common string methods, got: {:?}",
        completions
    );
}

#[test]
fn test_completer_trait_list_methods() {
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // Test list method completion
    let input = "[1, 2, 3].";
    let pos = input.len();
    
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Completer should not fail on list input");
    
    let (start, pairs) = result.unwrap();
    let completions: Vec<String> = pairs.into_iter().map(|p| p.replacement).collect();
    
    assert!(
        completions.iter().any(|s| s.contains("len") || s.contains("push") || s.contains("filter")),
        "Should suggest common list methods, got: {:?}",
        completions
    );
}

#[test]
fn test_completer_trait_partial_input() {
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // Test partial method name completion
    let input = "\"hello\".le";
    let pos = input.len();
    
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Completer should handle partial input");
    
    let (start, pairs) = result.unwrap();
    let completions: Vec<String> = pairs.into_iter().map(|p| p.replacement).collect();
    
    // Should suggest "len" for partial "le"
    assert!(
        completions.iter().any(|s| s.contains("len")),
        "Should suggest 'len' for partial input 'le', got: {:?}",
        completions
    );
}

#[test] 
fn test_completer_trait_help_queries() {
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // Test help query completion
    let input = "help(";
    let pos = input.len();
    
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Completer should handle help queries");
    
    let (start, pairs) = result.unwrap();
    assert!(!pairs.is_empty(), "Should return help suggestions");
}

#[test]
fn test_completer_trait_no_infinite_recursion() {
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // This test ensures the bug where complete() creates new RuchyCompleter
    // doesn't cause infinite recursion or other issues
    let input = "test.";
    let pos = input.len();
    
    // Should not hang or crash
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Completer should not hang or crash");
}

#[test]
fn test_cache_consistency_bug() {
    // This test reproduces the critical bug: creating new completer instances
    // loses cache state and causes inconsistent behavior
    
    let mut completer1 = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // First call - should populate cache
    let _result1 = completer1.complete("\"test\".", 7, &ctx);
    
    // The bug: complete() method creates a new completer, losing cache state
    // This should use the SAME completer instance, not create a new one
    let result2 = completer1.complete("\"test\".", 7, &ctx);
    assert!(result2.is_ok(), "Second call should work with same cache");
    
    // Verify we get consistent results
    let (_, pairs) = result2.unwrap();
    assert!(!pairs.is_empty(), "Should return cached completions");
}

#[test]
fn test_integration_with_repl_complete_method() {
    // Test that REPL's complete() method works properly
    use ruchy::runtime::repl::Repl;
    
    let repl = Repl::new().unwrap();
    
    // Test REPL completion integration
    let completions = repl.complete("\"hello\".");
    assert!(!completions.is_empty(), "REPL complete should return suggestions");
    
    assert!(
        completions.iter().any(|s| s.contains("len") || s.contains("split")),
        "REPL should suggest string methods, got: {:?}",
        completions
    );
}