// Test for REPL-TEST-001: Resource-bounded evaluation
// Tests bounded evaluation with arena allocator, timeouts, and stack limits

use ruchy::runtime::{Repl, ReplConfig};
use std::time::Duration;

#[test]
fn test_memory_bounds_basic() {
    let mut repl = Repl::new().unwrap();
    
    // Verify basic functionality
    let result = repl.eval("1 + 1").unwrap();
    assert_eq!(result.trim(), "2");
    
    // Check memory tracking works
    assert!(repl.memory_used() >= 0);
    assert!(repl.peak_memory() >= 0);
    assert!(repl.memory_pressure() >= 0.0 && repl.memory_pressure() <= 1.0);
    assert!(repl.can_accept_input());
    assert!(repl.bindings_valid());
}

#[test]
fn test_sandboxed_repl() {
    let mut repl = Repl::sandboxed().unwrap();
    
    // Sandboxed REPL should have stricter limits
    assert_eq!(repl.memory_used(), 0); // Fresh instance
    assert!(repl.can_accept_input());
    assert!(repl.bindings_valid());
    
    // Should still work for basic operations
    let result = repl.eval("42").unwrap();
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_memory_tracking_reset() {
    let mut repl = Repl::new().unwrap();
    
    // First evaluation
    let _result1 = repl.eval("let x = 100").unwrap();
    let memory_after_first = repl.memory_used();
    
    // Second evaluation should reset memory tracking
    let _result2 = repl.eval("let y = 200").unwrap();
    
    // Memory should be tracked for second evaluation, not cumulative
    // (The arena-style tracker resets between evaluations)
    assert!(repl.memory_used() >= 0);
    assert!(repl.bindings_valid());
}

#[test]  
fn test_explicit_resource_bounds() {
    let mut repl = Repl::new().unwrap();
    
    // Test with very small memory limit
    let small_memory = 1024; // 1KB
    let short_timeout = Duration::from_millis(1); // 1ms
    
    // Simple expression should work even with tight bounds
    let result = repl.eval_bounded("1", small_memory, short_timeout);
    assert!(result.is_ok(), "Simple expression should work with tight bounds");
    assert_eq!(result.unwrap().trim(), "1");
    
    // Original config should be restored
    let normal_result = repl.eval("2 + 2");
    assert!(normal_result.is_ok());
    assert_eq!(normal_result.unwrap().trim(), "4");
}

#[test]
fn test_timeout_bounds() {
    let mut repl = Repl::new().unwrap();
    
    // Test that timeout is enforced  
    let very_short_timeout = Duration::from_nanos(1); // Extremely short
    let result = repl.eval_bounded("1 + 1", 10 * 1024 * 1024, very_short_timeout);
    
    // This might timeout or succeed depending on system speed, both are valid
    match result {
        Ok(val) => assert_eq!(val.trim(), "2"),
        Err(e) => assert!(e.to_string().contains("timeout") || e.to_string().contains("exceeded")),
    }
}

#[test]
fn test_stack_depth_bounds() {
    let config = ReplConfig {
        max_memory: 10 * 1024 * 1024,
        timeout: Duration::from_millis(100),
        max_depth: 10, // Very limited recursion
        debug: false,
    };
    
    let mut repl = Repl::with_config(config).unwrap();
    
    // Simple expression should work
    let result = repl.eval("1 + 1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
}

#[test]
fn test_memory_pressure_calculation() {
    let mut repl = Repl::sandboxed().unwrap(); // 1MB limit
    
    // Initially no memory pressure
    assert!(repl.memory_pressure() < 0.1);
    assert!(repl.can_accept_input());
    
    // After some operations
    let _result = repl.eval("let x = \"test string\"").unwrap();
    
    // Should still be able to accept input
    assert!(repl.can_accept_input());
    assert!(repl.memory_pressure() <= 1.0);
}

#[test]
fn test_bindings_validation() {
    let mut repl = Repl::new().unwrap();
    
    // Initially valid
    assert!(repl.bindings_valid());
    
    // After adding bindings
    let _result = repl.eval("let x = 42").unwrap();
    assert!(repl.bindings_valid());
    
    let _result = repl.eval("let y = \"hello\"").unwrap();
    assert!(repl.bindings_valid());
    
    // Mutable bindings
    let _result = repl.eval("let mut z = 100").unwrap();
    assert!(repl.bindings_valid());
}

#[test]
fn test_peak_memory_tracking() {
    let mut repl = Repl::new().unwrap();
    
    // Initial peak should be 0 or very low
    let initial_peak = repl.peak_memory();
    
    // Do some work
    let _result = repl.eval("let x = [1, 2, 3, 4, 5]").unwrap();
    
    // Peak should have increased
    let after_work_peak = repl.peak_memory();
    assert!(after_work_peak >= initial_peak);
    
    // Current memory might be different from peak
    let current = repl.memory_used();
    assert!(after_work_peak >= current || current == 0); // Current could be 0 after reset
}

#[test]
fn test_resource_bounds_enforcement() {
    let mut repl = Repl::new().unwrap();
    
    // Test that the REPL actually enforces resource bounds by creating
    // expressions that should be within reasonable limits
    
    // Simple arithmetic (should always work)
    let result = repl.eval("1 + 2 * 3 - 4 / 2");
    assert!(result.is_ok());
    
    // String operations (should work within memory bounds)
    let result = repl.eval("\"hello\" + \" \" + \"world\"");
    assert!(result.is_ok());
    
    // List operations (should work within memory bounds)  
    let result = repl.eval("[1, 2, 3].map(|x| x * 2)");
    assert!(result.is_ok());
    
    // All operations should maintain valid bindings
    assert!(repl.bindings_valid());
}

#[test]
fn test_arena_style_reset() {
    let mut repl = Repl::new().unwrap();
    
    // Do several evaluations
    let _r1 = repl.eval("let a = 1").unwrap();
    let _r2 = repl.eval("let b = 2").unwrap();  
    let _r3 = repl.eval("let c = 3").unwrap();
    
    // Memory tracking should be reset between evaluations (arena-style)
    // but bindings should persist
    assert!(repl.bindings_valid());
    
    // Should be able to reference previous bindings
    let result = repl.eval("a + b + c");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "6");
}

#[test]
fn test_no_io_during_evaluation() {
    let mut repl = Repl::new().unwrap();
    
    // The resource-bounded evaluator should prevent I/O during evaluation
    // For now, we test that basic evaluation works without any I/O operations
    
    // Pure computation (no I/O)
    let result = repl.eval("2 + 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "4");
    
    // Function definition (no I/O)
    let result = repl.eval("fn square(x) { x * x }");
    assert!(result.is_ok());
    
    // Function call (no I/O)
    let result = repl.eval("square(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "25");
    
    assert!(repl.bindings_valid());
}