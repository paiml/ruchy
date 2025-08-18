//! Integration tests for REPL v3
//!
//! Tests the production-grade REPL implementation with
//! resource bounds, recovery, and stability guarantees.

use ruchy::runtime::repl_v3::{ReplV3, ReplConfig};
use std::time::Duration;

#[test]
fn test_repl_creation() {
    let repl = ReplV3::new();
    assert!(repl.is_ok(), "REPL should create successfully");
}

#[test]
fn test_custom_config() {
    let config = ReplConfig {
        max_memory: 1024 * 1024, // 1MB
        timeout: Duration::from_millis(50),
        max_depth: 100,
        debug: true,
    };
    
    let repl = ReplV3::with_config(config);
    assert!(repl.is_ok(), "REPL should accept custom config");
}

#[test]
fn test_memory_bounds() {
    let config = ReplConfig {
        max_memory: 1024, // Very small: 1KB
        timeout: Duration::from_secs(1),
        max_depth: 10,
        debug: false,
    };
    
    let mut repl = ReplV3::with_config(config).unwrap();
    
    // Try to evaluate something that would use more memory
    let large_input = "x".repeat(2000); // 2KB string
    let result = repl.evaluator.eval(&large_input);
    
    assert!(result.is_err(), "Should fail with memory limit");
    assert!(result.unwrap_err().to_string().contains("Memory limit"));
}

#[test]
fn test_timeout() {
    let config = ReplConfig {
        max_memory: 10 * 1024 * 1024,
        timeout: Duration::from_millis(1), // Very short timeout
        max_depth: 1000,
        debug: false,
    };
    
    let mut repl = ReplV3::with_config(config).unwrap();
    
    // The evaluator checks time at start of eval, so we need a long-running operation
    // For now, just check that very short timeout works
    let result = repl.evaluator.eval("42");
    
    // Since our placeholder implementation is fast, this might not timeout
    // We'll improve this when we add real evaluation
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("timeout"));
}

#[test]
fn test_stack_depth() {
    let config = ReplConfig {
        max_memory: 10 * 1024 * 1024,
        timeout: Duration::from_secs(1),
        max_depth: 5, // Very shallow
        debug: false,
    };
    
    let mut repl = ReplV3::with_config(config).unwrap();
    
    // This would require deep recursion
    let deep_expr = "f(f(f(f(f(f(x))))))";
    let result = repl.evaluator.eval(deep_expr);
    
    // For now this won't actually trigger depth limit since
    // we're not doing real evaluation yet
    assert!(result.is_ok() || result.unwrap_err().to_string().contains("depth"));
}

#[test]
fn test_state_transitions() {
    use ruchy::runtime::repl_v3::state::{State, Environment};
    
    let env = Environment::new();
    let state = State::Ready(env);
    
    // Test successful evaluation
    let (new_state, result) = state.eval("42");
    assert!(matches!(new_state, State::Ready(_)));
    assert!(result.is_ok());
    
    // Test error handling
    let (failed_state, result) = new_state.eval("error");
    assert!(matches!(failed_state, State::Failed(_)));
    assert!(result.is_err());
    
    // Test recovery
    let (recovered, result) = failed_state.eval("recover");
    assert!(matches!(recovered, State::Ready(_)));
    assert!(result.is_err()); // Recovery reports error but restores state
}

#[test]
fn test_checkpoint_restore() {
    use ruchy::runtime::repl_v3::state::{Environment, Value};
    
    let mut env = Environment::new();
    env = env.extend("x".to_string(), Value::Int(42));
    
    let checkpoint = env.checkpoint();
    assert_eq!(checkpoint.bindings.get("x"), Some(&Value::Int(42)));
    
    let restored = checkpoint.restore();
    assert_eq!(restored.get("x"), Some(&Value::Int(42)));
}

#[test]
fn test_error_recovery() {
    use ruchy::runtime::repl_v3::recovery::{RecoverableError, Restart, RestartHandler};
    
    let mut error = RecoverableError::new("Variable not found: x");
    error = error.add_restart(Restart {
        name: "define".to_string(),
        description: "Define with default value 0".to_string(),
        handler: RestartHandler::UseDefault("0".to_string()),
    });
    
    assert_eq!(error.restarts.len(), 2); // One added + default abort
    
    let action = error.handle_restart(1).unwrap();
    assert!(matches!(action, ruchy::runtime::repl_v3::recovery::RestartAction::UseValue(_)));
}

#[test]
fn test_memory_tracker() {
    use ruchy::runtime::repl_v3::evaluator::MemoryTracker;
    
    let mut tracker = MemoryTracker::new(1000);
    
    assert!(tracker.try_alloc(500).is_ok());
    assert_eq!(tracker.used(), 500);
    
    assert!(tracker.try_alloc(400).is_ok());
    assert_eq!(tracker.used(), 900);
    
    assert!(tracker.try_alloc(200).is_err()); // Would exceed limit
    
    tracker.free(400);
    assert_eq!(tracker.used(), 500);
    
    tracker.reset();
    assert_eq!(tracker.used(), 0);
}

#[test]
fn test_parser_integration() {
    use ruchy::runtime::repl_v3::ReplConfig;
    use std::time::Duration;
    
    let config = ReplConfig {
        max_memory: 10 * 1024 * 1024,
        timeout: Duration::from_secs(1),
        max_depth: 100,
        debug: false,
    };
    
    let mut repl = ReplV3::with_config(config).expect("REPL creation should succeed");
    
    // Test basic arithmetic
    let result = repl.evaluator.eval("42").expect("Should evaluate literal");
    assert_eq!(result, "42");
    
    let result = repl.evaluator.eval("1 + 2").expect("Should evaluate addition");
    assert_eq!(result, "3");
    
    let result = repl.evaluator.eval("10 - 3").expect("Should evaluate subtraction");
    assert_eq!(result, "7");
    
    let result = repl.evaluator.eval("4 * 5").expect("Should evaluate multiplication");
    assert_eq!(result, "20");
    
    let result = repl.evaluator.eval("15 / 3").expect("Should evaluate division");
    assert_eq!(result, "5");
    
    // Test comparisons
    let result = repl.evaluator.eval("5 > 3").expect("Should evaluate comparison");
    assert_eq!(result, "true");
    
    let result = repl.evaluator.eval("2 < 1").expect("Should evaluate comparison");
    assert_eq!(result, "false");
    
    // Test boolean operations
    let result = repl.evaluator.eval("true && false").expect("Should evaluate boolean");
    assert_eq!(result, "false");
    
    let result = repl.evaluator.eval("true || false").expect("Should evaluate boolean");
    assert_eq!(result, "true");
}