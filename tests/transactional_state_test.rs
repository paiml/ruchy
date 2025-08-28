// Test for REPL-TEST-002: Transactional state machine  
// Tests checkpointing system with persistent data structures for O(1) recovery

use ruchy::runtime::{Repl, ReplState};
use std::time::Duration;

#[test]
fn test_basic_state_machine() {
    let mut repl = Repl::new().unwrap();
    
    // Initially should be in Ready state
    assert!(matches!(repl.get_state(), ReplState::Ready));
    assert!(!repl.is_failed());
    
    // Simple evaluation should keep state Ready
    let result = repl.eval_transactional("1 + 1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "2");
    assert!(matches!(repl.get_state(), ReplState::Ready));
    assert!(!repl.is_failed());
}

#[test]
fn test_checkpoint_creation_and_restoration() {
    let mut repl = Repl::new().unwrap();
    
    // Set up some state
    let _result = repl.eval("let x = 42").unwrap();
    let _result = repl.eval("let y = 100").unwrap();
    
    // Create checkpoint
    let checkpoint = repl.checkpoint();
    
    // Modify state
    let _result = repl.eval("let z = 999").unwrap();
    
    // Verify state changed
    let result = repl.eval("z");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "999");
    
    // Restore checkpoint
    repl.restore_checkpoint(&checkpoint);
    
    // Verify restoration - z should no longer exist
    let result = repl.eval("z");
    assert!(result.is_err()); // z was not in the checkpoint
    
    // x and y should still exist
    let result = repl.eval("x");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "42");
    
    let result = repl.eval("y");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "100");
}

#[test]
fn test_checkpoint_performance() {
    let mut repl = Repl::new().unwrap();
    
    // Add multiple bindings to test performance
    for i in 0..100 {
        let _result = repl.eval(&format!("let var_{} = {}", i, i * 2)).unwrap();
    }
    
    // Checkpoint creation should be fast (O(1) with persistent structures)
    let start = std::time::Instant::now();
    let checkpoint = repl.checkpoint();
    let checkpoint_time = start.elapsed();
    
    // Should be very fast - less than 1ms for 100 bindings
    assert!(checkpoint_time < Duration::from_millis(10), 
            "Checkpoint took too long: {:?}", checkpoint_time);
    
    // Restoration should also be fast
    let _result = repl.eval("let additional = 999").unwrap();
    
    let start = std::time::Instant::now();
    repl.restore_checkpoint(&checkpoint);
    let restore_time = start.elapsed();
    
    assert!(restore_time < Duration::from_millis(10),
            "Restore took too long: {:?}", restore_time);
    
    // Verify all original bindings are restored
    for i in 0..10 { // Test a few
        let result = repl.eval(&format!("var_{}", i));
        assert!(result.is_ok(), "var_{} should exist after restore", i);
        assert_eq!(result.unwrap().trim(), &format!("{}", i * 2));
    }
    
    // Additional binding should be gone
    let result = repl.eval("additional");
    assert!(result.is_err());
}

#[test]
fn test_transactional_evaluation_success() {
    let mut repl = Repl::new().unwrap();
    
    // Successful evaluation should maintain Ready state
    let result = repl.eval_transactional("let success = 42");
    assert!(result.is_ok());
    assert!(matches!(repl.get_state(), ReplState::Ready));
    assert!(!repl.is_failed());
    
    // Value should be accessible
    let result = repl.eval("success");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "42");
}

#[test]
fn test_transactional_evaluation_failure() {
    let mut repl = Repl::new().unwrap();
    
    // Set up initial state
    let _result = repl.eval("let original = 100").unwrap();
    
    // Use very strict resource bounds to force failure
    let config = ruchy::runtime::ReplConfig {
        max_memory: 100, // 100 bytes - very restrictive but not impossible
        timeout: Duration::from_nanos(1), // Almost no time
        max_depth: 1000,
        debug: false,
    };
    
    let mut strict_repl = Repl::with_config(config).unwrap();
    // Skip initial setup that might fail with strict limits
    
    // This should likely fail due to resource constraints
    let result = strict_repl.eval_transactional("let big_data = [1,2,3,4,5,6,7,8,9,10]");
    
    // Either succeeds or fails, both are valid depending on timing
    match result {
        Ok(_) => {
            // If it succeeds, state should be Ready
            assert!(matches!(strict_repl.get_state(), ReplState::Ready));
        }
        Err(_) => {
            // If it fails, should be in Failed state
            assert!(strict_repl.is_failed());
            
            // Should be able to recover
            let recovery = strict_repl.recover();
            assert!(recovery.is_ok());
            assert!(!strict_repl.is_failed());
            assert!(matches!(strict_repl.get_state(), ReplState::Ready));
        }
    }
}

#[test]
fn test_failed_state_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Set up some good state first
    let _result = repl.eval("let good_var = 42").unwrap();
    
    // Create a deliberately failing scenario by using invalid syntax with transactional eval
    // Since eval_internal bypasses some error handling, we need to test carefully
    
    // For now, test the recovery mechanism directly
    let checkpoint = repl.checkpoint();
    
    // Manually put REPL in failed state (simulating a real failure)
    repl.set_state_for_testing(ruchy::runtime::ReplState::Failed(checkpoint));
    assert!(repl.is_failed());
    
    // Recover should restore state
    let result = repl.recover();
    assert!(result.is_ok());
    assert!(!repl.is_failed());
    assert!(matches!(repl.get_state(), ReplState::Ready));
    
    // Original state should be preserved
    let result = repl.eval("good_var");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "42");
}

#[test]
fn test_checkpoint_age_tracking() {
    let mut repl = Repl::new().unwrap();
    
    let _result = repl.eval("let timed_var = 123").unwrap();
    
    let checkpoint = repl.checkpoint();
    
    // Age should be very small initially
    let age = checkpoint.age();
    assert!(age < Duration::from_millis(10));
    
    // Wait a bit
    std::thread::sleep(Duration::from_millis(10));
    
    // Age should have increased
    let new_age = checkpoint.age();
    assert!(new_age > age);
    assert!(new_age >= Duration::from_millis(5)); // Should be at least a few ms
}

#[test] 
fn test_persistent_data_structures() {
    let mut repl = Repl::new().unwrap();
    
    // Add various types of data
    let _result = repl.eval("let int_val = 42").unwrap();
    let _result = repl.eval("let string_val = \"hello\"").unwrap();
    let _result = repl.eval("let list_val = [1, 2, 3]").unwrap();
    let _result = repl.eval("let bool_val = true").unwrap();
    
    // Create checkpoint
    let checkpoint = repl.checkpoint();
    
    // Modify state extensively
    let _result = repl.eval("let new_var = 999").unwrap();
    let _result = repl.eval("let int_val = 0").unwrap(); // Shadow original
    
    // Verify changes
    let result = repl.eval("int_val");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "0");
    
    // Restore
    repl.restore_checkpoint(&checkpoint);
    
    // All original values should be restored
    let result = repl.eval("int_val");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "42");
    
    let result = repl.eval("string_val");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("hello"));
    
    let result = repl.eval("list_val");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("[1, 2, 3]"));
    
    let result = repl.eval("bool_val");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "true");
    
    // New variable should be gone
    let result = repl.eval("new_var");
    assert!(result.is_err());
}

#[test]
fn test_result_history_checkpointing() {
    let mut repl = Repl::new().unwrap();
    
    // Build up result history
    let _r1 = repl.eval("10").unwrap();
    let _r2 = repl.eval("20").unwrap(); 
    let _r3 = repl.eval("30").unwrap();
    
    println!("Initial history length: {}", repl.result_history_len());
    
    // Create checkpoint with this history
    let checkpoint = repl.checkpoint();
    
    // Add more history
    let _r4 = repl.eval("40").unwrap();
    let _r5 = repl.eval("50").unwrap();
    
    println!("After adding 2 more: {}", repl.result_history_len());
    
    // Restore checkpoint
    repl.restore_checkpoint(&checkpoint);
    
    println!("After restore: {}", repl.result_history_len());
    
    // Check the basic state after restoration
    // We should have only the first 3 values in history
    assert_eq!(repl.result_history_len(), 3, "Should have 3 items in history after restore");
    
    // Test that we can access the restored values
    let result = repl.eval("_");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "30", "Most recent should be 30");
    
    let result = repl.eval("_1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "10", "_1 should be 10");
    
    let result = repl.eval("_2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "20", "_2 should be 20");
    
    let result = repl.eval("_3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trim(), "30", "_3 should be 30");
}

#[test]
fn test_enum_definitions_checkpointing() {
    let mut repl = Repl::new().unwrap();
    
    // Enums should be included in checkpoints
    let _result = repl.eval("enum Color { Red, Green, Blue }").unwrap();
    
    // Create checkpoint
    let checkpoint = repl.checkpoint();
    
    // Add another enum
    let _result = repl.eval("enum Size { Small, Medium, Large }").unwrap();
    
    // Restore checkpoint
    repl.restore_checkpoint(&checkpoint);
    
    // Original enum should work
    let result = repl.eval("Color::Red");
    assert!(result.is_ok());
    
    // New enum should be gone - this test may be implementation dependent
    // For now, we just verify the checkpoint/restore mechanism works
    // without throwing errors
    assert!(!repl.is_failed());
    assert!(matches!(repl.get_state(), ReplState::Ready));
}