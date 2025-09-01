// TDD Test for Terminal Tab Completion Issues
// Reproducing the specific issue users are reporting

use ruchy::runtime::completion::RuchyCompleter;
use ruchy::runtime::repl::Repl;
use rustyline::completion::Completer;
use rustyline::history::DefaultHistory;
use rustyline::Context;

#[test]
fn test_terminal_tab_completion_issue() {
    // Reproduce the exact user-reported scenario
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // Test the exact scenario users are reporting
    let test_cases = vec![
        ("\"hello\".", 8),     // String method completion
        ("[1,2,3].", 8),       // List method completion  
        ("print", 5),          // Function name completion
        ("help(", 5),          // Help completion
    ];
    
    for (input, pos) in test_cases {
        println!("Testing completion for: '{}' at position {}", input, pos);
        
        let result = completer.complete(input, pos, &ctx);
        assert!(result.is_ok(), "Completer failed for input: '{}'", input);
        
        let (_start, pairs) = result.unwrap();
        println!("Got {} completions for '{}'", pairs.len(), input);
        
        if !pairs.is_empty() {
            let first_five: Vec<String> = pairs.iter()
                .take(5)
                .map(|p| p.replacement.clone())
                .collect();
            println!("  Sample completions: {:?}", first_five);
        }
        
        // The bug: if no completions are returned, tab completion appears "broken"
        if pairs.is_empty() {
            println!("  ❌ WARNING: No completions returned for '{}'", input);
        } else {
            println!("  ✅ OK: {} completions returned for '{}'", pairs.len(), input);
        }
    }
}

#[test]  
fn test_completer_caching_bug() {
    // This tests the critical bug in the Completer implementation
    // The issue: complete() creates a new RuchyCompleter, losing state
    
    println!("Testing the caching bug...");
    
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // First call
    let result1 = completer.complete("\"test\".", 7, &ctx);
    assert!(result1.is_ok(), "First call should work");
    let (_start1, pairs1) = result1.unwrap();
    println!("First call returned {} completions", pairs1.len());
    
    // Second call - should be consistent
    let result2 = completer.complete("\"test\".", 7, &ctx);
    assert!(result2.is_ok(), "Second call should work");
    let (_start2, pairs2) = result2.unwrap();  
    println!("Second call returned {} completions", pairs2.len());
    
    // BUG: Because complete() creates new RuchyCompleter, results might be inconsistent
    // The cache state is lost each time
    assert_eq!(pairs1.len(), pairs2.len(), "Results should be consistent between calls");
}

#[test]
fn test_repl_complete_method() {
    // Test the REPL's complete method which users actually interact with
    println!("Testing REPL complete method...");
    
    let repl = Repl::new().unwrap();
    
    let test_inputs = vec![
        "\"hello\".",
        "[1,2,3].",
        "print",
        "help(",
        "len",
    ];
    
    for input in test_inputs {
        println!("Testing REPL completion for: '{}'", input);
        
        let completions = repl.complete(input);
        println!("  Got {} completions", completions.len());
        
        if completions.is_empty() {
            println!("  ❌ WARNING: REPL returned no completions for '{}'", input);
        } else {
            let first_three: Vec<String> = completions.iter()
                .take(3)
                .cloned()
                .collect();
            println!("  ✅ OK: Sample completions: {:?}", first_three);
        }
        
        // This is what users experience - if empty, tab completion "doesn't work"
        if input == "\"hello\"." {
            assert!(!completions.is_empty(), "String methods should always return completions");
        }
    }
}

#[test]
fn test_specific_completion_bug_reproduction() {
    // Reproduce the exact issue: Completer::complete creates new instance
    println!("Testing the specific bug in Completer::complete...");
    
    let completer = RuchyCompleter::new();
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    
    // This is the problematic code from completion.rs:
    // ```rust
    // fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
    //     let context = self.analyze_context(line, pos);
    //     let mut completer = RuchyCompleter::new();  // ❌ BUG: Creates new instance!
    //     let completions = completer.complete_context(context);
    // ```
    
    // The fix should use self instead of creating new instance:
    // ```rust
    // fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
    //     let context = self.analyze_context(line, pos);
    //     let completions = self.complete_context(context);  // ✅ Use self
    // ```
    
    let input = "\"hello\".le";
    let pos = input.len();
    
    let result = completer.complete(input, pos, &ctx);
    assert!(result.is_ok(), "Should complete successfully");
    
    let (_start, pairs) = result.unwrap();
    println!("Completions for '{}': {}", input, pairs.len());
    
    // Look for "len" completion specifically
    let has_len = pairs.iter().any(|p| p.replacement.contains("len"));
    if has_len {
        println!("  ✅ Found 'len' completion as expected");  
    } else {
        println!("  ❌ Missing 'len' completion - potential bug");
        println!("  Available completions: {:?}", 
                 pairs.iter().map(|p| &p.replacement).collect::<Vec<_>>());
    }
}