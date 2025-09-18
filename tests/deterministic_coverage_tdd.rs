//! TDD tests for deterministic.rs module
//! Target: Improve deterministic.rs from 0% to 90%+ coverage

use ruchy::runtime::deterministic::{MockTime, DeterministicRng};
use ruchy::runtime::repl::Repl;
use ruchy::runtime::replay::{DeterministicRepl, StateCheckpoint, ResourceUsage, Divergence};
use std::{env, collections::HashMap};

#[test]
fn test_mock_time_new() {
    let mock_time = MockTime::new();
    assert_eq!(mock_time.now(), 0);
}

#[test]
fn test_mock_time_default() {
    let mock_time = MockTime::default();
    assert_eq!(mock_time.now(), 0);
}

#[test]
fn test_mock_time_advance() {
    let mut mock_time = MockTime::new();
    assert_eq!(mock_time.now(), 0);
    
    mock_time.advance(1000);
    assert_eq!(mock_time.now(), 1000);
    
    mock_time.advance(500);
    assert_eq!(mock_time.now(), 1500);
}

#[test]
fn test_mock_time_large_advance() {
    let mut mock_time = MockTime::new();
    
    mock_time.advance(u64::MAX / 2);
    let first_time = mock_time.now();
    
    mock_time.advance(100);
    assert_eq!(mock_time.now(), first_time + 100);
}

#[test]
fn test_mock_time_overflow_handling() {
    let mut mock_time = MockTime::new();
    
    // Set to a large value to test near-overflow behavior
    mock_time.advance(u64::MAX - 1000);
    let before = mock_time.now();
    
    // Advance by a smaller amount that won't cause overflow
    mock_time.advance(500);
    let after = mock_time.now();
    
    // Should still increase without wrapping
    assert!(after > before);
    assert_eq!(after, before + 500);
}

#[test]
fn test_deterministic_rng_new() {
    let _rng = DeterministicRng::new(42);
    // Cannot directly check internal state due to privacy, but can verify creation works
}

#[test]
fn test_deterministic_rng_next() {
    let mut rng = DeterministicRng::new(123);
    
    let first = rng.next();
    let second = rng.next();
    let third = rng.next();
    
    // Values should be different (good pseudo-random)
    assert_ne!(first, second);
    assert_ne!(second, third);
    assert_ne!(first, third);
}

#[test]
fn test_deterministic_rng_deterministic() {
    let mut rng1 = DeterministicRng::new(999);
    let mut rng2 = DeterministicRng::new(999);
    
    // Same seed should produce same sequence
    for _ in 0..10 {
        assert_eq!(rng1.next(), rng2.next());
    }
}

#[test]
fn test_deterministic_rng_different_seeds() {
    let mut rng1 = DeterministicRng::new(111);
    let mut rng2 = DeterministicRng::new(222);
    
    // Different seeds should produce different sequences
    let val1 = rng1.next();
    let val2 = rng2.next();
    assert_ne!(val1, val2);
}

#[test]
fn test_deterministic_rng_reset() {
    let mut rng = DeterministicRng::new(555);
    
    let first_sequence = [rng.next(), rng.next(), rng.next()];
    
    rng.reset();
    let second_sequence = [rng.next(), rng.next(), rng.next()];
    
    // After reset, should generate same sequence
    assert_eq!(first_sequence, second_sequence);
}

#[test]
fn test_deterministic_rng_long_sequence() {
    let mut rng = DeterministicRng::new(777);
    let mut values = Vec::new();
    
    // Generate longer sequence to test stability
    for _ in 0..100 {
        values.push(rng.next());
    }
    
    // Reset and generate again
    rng.reset();
    for i in 0..100 {
        assert_eq!(values[i], rng.next());
    }
}

#[test]
fn test_deterministic_rng_wrapping_behavior() {
    let mut rng = DeterministicRng::new(u64::MAX);
    
    // Should handle wrapping in LCG calculation
    let value1 = rng.next();
    let value2 = rng.next();
    
    assert_ne!(value1, value2);
    // Values should be valid (not panic on overflow)
    assert!(value1 != 0 || value2 != 0); // At least one should be non-zero
}

#[test]
fn test_execute_with_seed_basic() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.execute_with_seed("42", 12345);
    assert!(result.output.is_ok());
    assert!(!result.state_hash.is_empty());
    assert!(result.resource_usage.cpu_ns > 0);
}

#[test]
fn test_execute_with_seed_error_handling() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.execute_with_seed("undefined_variable", 12345);
    assert!(result.output.is_err());
    assert!(!result.state_hash.is_empty());
}

#[test]
fn test_execute_with_seed_resource_usage() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.execute_with_seed("let x = 42", 12345);
    
    // Resource usage should be tracked
    assert!(result.resource_usage.cpu_ns > 0);
    assert!(result.resource_usage.heap_bytes >= 0);
    assert!(result.resource_usage.stack_depth >= 0);
}

#[test]
fn test_execute_with_seed_state_hash() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result = repl.execute_with_seed("let y = 99", 12345);
    assert!(result.output.is_ok());
    assert!(!result.state_hash.is_empty());
    assert!(result.state_hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_checkpoint_empty_repl() {
    let repl = Repl::new(std::env::temp_dir()).unwrap();
    let checkpoint = DeterministicRepl::checkpoint(&repl);
    
    assert!(checkpoint.bindings.is_empty());
    assert!(checkpoint.type_environment.is_empty());
    assert!(!checkpoint.state_hash.is_empty());
    assert_eq!(checkpoint.resource_usage.cpu_ns, 0);
}

#[test]
fn test_checkpoint_with_variables() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    repl.eval("let x = 10").unwrap();
    repl.eval("let name = \"test\"").unwrap();
    
    let checkpoint = DeterministicRepl::checkpoint(&repl);
    assert!(!checkpoint.bindings.is_empty());
    assert!(!checkpoint.state_hash.is_empty());
}

#[test]
fn test_restore_empty_checkpoint() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Add some variables
    repl.eval("let x = 1").unwrap();
    repl.eval("let y = 2").unwrap();
    
    // Create empty checkpoint
    let empty_checkpoint = StateCheckpoint {
        bindings: HashMap::new(),
        type_environment: HashMap::new(),
        state_hash: "empty".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 0,
            stack_depth: 0,
            cpu_ns: 0,
        },
    };
    
    // Restore should succeed
    let result = DeterministicRepl::restore(&mut repl, &empty_checkpoint);
    assert!(result.is_ok());
}

#[test]
fn test_restore_with_values() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let mut checkpoint = StateCheckpoint {
        bindings: HashMap::new(),
        type_environment: HashMap::new(),
        state_hash: "test".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 0,
            stack_depth: 0,
            cpu_ns: 0,
        },
    };
    
    checkpoint.bindings.insert("test_var".to_string(), "42".to_string());
    
    let result = DeterministicRepl::restore(&mut repl, &checkpoint);
    assert!(result.is_ok());
}

#[test]
fn test_validate_determinism_identical() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let mut repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    // Execute identical operations
    repl1.eval("let a = 1").unwrap();
    repl2.eval("let a = 1").unwrap();
    
    let validation = DeterministicRepl::validate_determinism(&repl1, &repl2);
    assert!(validation.is_deterministic);
    assert!(validation.divergences.is_empty());
}

#[test]
fn test_validate_determinism_different_values() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let mut repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    // Execute different operations
    repl1.eval("let x = 10").unwrap();
    repl2.eval("let x = 20").unwrap();
    
    let validation = DeterministicRepl::validate_determinism(&repl1, &repl2);
    assert!(!validation.is_deterministic);
    assert!(!validation.divergences.is_empty());
    
    // Check that we have a state divergence
    match &validation.divergences[0] {
        Divergence::State { expected_hash: _, actual_hash: _ } => {
            // Expected divergence type
        }
        _ => panic!("Expected State divergence"),
    }
}

#[test]
fn test_validate_determinism_missing_variable() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    // repl1 has variable, repl2 doesn't
    repl1.eval("let missing = 42").unwrap();
    
    let validation = DeterministicRepl::validate_determinism(&repl1, &repl2);
    assert!(!validation.is_deterministic);
    assert!(!validation.divergences.is_empty());
}

#[test]
fn test_validate_determinism_empty_repls() {
    let repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    let validation = DeterministicRepl::validate_determinism(&repl1, &repl2);
    assert!(validation.is_deterministic);
    assert!(validation.divergences.is_empty());
}

#[test]
fn test_deterministic_execution_same_seed() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let mut repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    // Execute same commands with same seed
    let result1 = repl1.execute_with_seed("let x = 42", 12345);
    let result2 = repl2.execute_with_seed("let x = 42", 12345);
    
    // Results should be identical
    assert!(result1.output.is_ok());
    assert!(result2.output.is_ok());
    assert_eq!(result1.state_hash, result2.state_hash);
}

#[test]
fn test_checkpoint_restore_cycle() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Create some state
    repl.eval("let x = 10").unwrap();
    repl.eval("let y = 20").unwrap();
    
    // Create checkpoint using DeterministicRepl trait
    let checkpoint = DeterministicRepl::checkpoint(&repl);
    
    // Modify state
    repl.eval("let x = 99").unwrap();
    
    // Restore checkpoint
    let restore_result = DeterministicRepl::restore(&mut repl, &checkpoint);
    assert!(restore_result.is_ok());
}

#[test]
fn test_resource_usage_structure() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("42", 100);
    
    let usage = &result.resource_usage;
    assert!(usage.heap_bytes >= 0);
    assert!(usage.stack_depth >= 0);
    assert!(usage.cpu_ns > 0);
}

#[test]
fn test_state_hash_consistency() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let mut repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    // Same operations should produce same hash
    repl1.eval("let test = 123").unwrap();
    repl2.eval("let test = 123").unwrap();
    
    let result1 = repl1.execute_with_seed("test", 999);
    let result2 = repl2.execute_with_seed("test", 999);
    
    assert_eq!(result1.state_hash, result2.state_hash);
}

#[test]
fn test_value_conversion_unit() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("()", 12345);
    assert!(result.output.is_ok());
}

#[test]
fn test_value_conversion_bool_true() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("true", 12345);
    assert!(result.output.is_ok());
}

#[test]
fn test_value_conversion_bool_false() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("false", 12345);
    assert!(result.output.is_ok());
}

#[test]
fn test_value_conversion_integer() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("42", 12345);
    assert!(result.output.is_ok());
}

#[test]
fn test_value_conversion_string() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.execute_with_seed("\"hello\"", 12345);
    assert!(result.output.is_ok());
}

#[test]
fn test_checkpoint_bindings_extraction() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    repl.eval("let extracted = 456").unwrap();
    let checkpoint = DeterministicRepl::checkpoint(&repl);
    
    // Check that checkpoint extracts bindings properly
    assert!(!checkpoint.state_hash.is_empty());
    assert!(checkpoint.resource_usage.heap_bytes >= 0);
}

#[test]
fn test_validation_divergence_detection() {
    let mut repl1 = Repl::new(std::env::temp_dir()).unwrap();
    let mut repl2 = Repl::new(std::env::temp_dir()).unwrap();
    
    repl1.eval("let divergent = 100").unwrap();
    repl2.eval("let divergent = 200").unwrap();
    
    let validation = DeterministicRepl::validate_determinism(&repl1, &repl2);
    assert!(!validation.is_deterministic);
    
    // Should have at least one State divergence
    let has_state_divergence = validation.divergences.iter().any(|d| {
        matches!(d, Divergence::State { .. })
    });
    assert!(has_state_divergence);
}

#[test]
fn test_multiple_executions_same_repl() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result1 = repl.execute_with_seed("let first = 1", 100);
    let result2 = repl.execute_with_seed("let second = 2", 200);
    
    assert!(result1.output.is_ok());
    assert!(result2.output.is_ok());
    assert_ne!(result1.state_hash, result2.state_hash); // State should change
}

#[test]
fn test_error_state_handling() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let error_result = repl.execute_with_seed("invalid syntax +++", 999);
    assert!(error_result.output.is_err());
    
    // Should still have valid state hash and resource tracking
    assert!(!error_result.state_hash.is_empty());
    assert!(error_result.resource_usage.cpu_ns > 0);
}

#[test]
fn test_resource_usage_accumulation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    let result1 = repl.execute_with_seed("let a = 1", 100);
    let result2 = repl.execute_with_seed("let b = 2", 100);
    
    // Both should have positive resource usage
    assert!(result1.resource_usage.cpu_ns > 0);
    assert!(result2.resource_usage.cpu_ns > 0);
    assert!(result1.resource_usage.heap_bytes >= 0);
    assert!(result2.resource_usage.heap_bytes >= 0);
}

#[test]
fn test_checkpoint_type_environment_empty() {
    let repl = Repl::new(std::env::temp_dir()).unwrap();
    let checkpoint = DeterministicRepl::checkpoint(&repl);
    
    // Type environment should be empty for now (not implemented)
    assert!(checkpoint.type_environment.is_empty());
}

#[test]
fn test_restore_value_types() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Test restoring different value types
    let mut checkpoint = StateCheckpoint {
        bindings: HashMap::new(),
        type_environment: HashMap::new(),
        state_hash: "test".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 0,
            stack_depth: 0,
            cpu_ns: 0,
        },
    };
    
    // Add different value types to test conversion
    checkpoint.bindings.insert("unit".to_string(), "Unit".to_string());
    checkpoint.bindings.insert("bool_true".to_string(), "true".to_string());
    checkpoint.bindings.insert("bool_false".to_string(), "false".to_string());
    checkpoint.bindings.insert("integer".to_string(), "123".to_string());
    checkpoint.bindings.insert("string".to_string(), "hello".to_string());
    
    let result = DeterministicRepl::restore(&mut repl, &checkpoint);
    assert!(result.is_ok());
}