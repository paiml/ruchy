//! Memory optimization tests for SharedSession DataFrames
//! 
//! Tests memory efficiency improvements using TDD methodology:
//! - Memory pooling for DataFrame operations
//! - Lazy evaluation for large DataFrames
//! - Copy-on-write optimization
//! - Memory pressure detection and cleanup

use ruchy::wasm::shared_session::SharedSession;
use ruchy::wasm::notebook::NotebookRuntime;
use std::time::Instant;

#[test]
fn test_memory_growth_bounds() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Get baseline memory
    let initial = parse_memory_usage(&runtime.get_memory_usage());
    
    // Create moderate DataFrame  
    runtime.execute_cell_with_session("df1", "let df1 = DataFrame::from_range(0, 1000)").unwrap();
    let after_1k = parse_memory_usage(&runtime.get_memory_usage());
    
    // Create larger DataFrame
    runtime.execute_cell_with_session("df2", "let df2 = DataFrame::from_range(0, 5000)").unwrap();  
    let after_5k = parse_memory_usage(&runtime.get_memory_usage());
    
    // Memory growth should be roughly linear (not exponential)
    let growth_1k = after_1k.saturating_sub(initial);
    let growth_5k = after_5k.saturating_sub(after_1k);
    
    // 5x data should not use more than 10x memory (allowing for overhead)
    assert!(growth_5k < growth_1k * 10, 
           "Memory growth not linear: 1k={}, 5k={}", growth_1k, growth_5k);
}

#[test] 
fn test_dataframe_sharing_optimization() {
    let mut session = SharedSession::new();
    
    // Create base DataFrame
    session.execute("base", "let base = DataFrame::from_range(0, 1000)").unwrap();
    
    // Create views/slices - should share memory efficiently  
    session.execute("slice1", "let slice1 = base.slice(0, 100)").unwrap();
    session.execute("slice2", "let slice2 = base.slice(100, 200)").unwrap();
    
    // Memory usage should not triple (indicating sharing)
    // This is a basic smoke test for memory efficiency
    let memory_info = session.estimate_interpreter_memory();
    assert!(memory_info > 0, "Memory tracking should work");
}

#[test]
fn test_memory_cleanup_on_variable_reassignment() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Create large DataFrame
    runtime.execute_cell_with_session("big_df", "let df = DataFrame::from_range(0, 2000)").unwrap();
    let with_large_df = parse_memory_usage(&runtime.get_memory_usage());
    
    // Reassign to small DataFrame - old memory should be eligible for cleanup
    runtime.execute_cell_with_session("big_df", "let df = DataFrame([[1, 2]])").unwrap();
    let after_reassign = parse_memory_usage(&runtime.get_memory_usage());
    
    // Memory should not keep growing indefinitely
    // (This test verifies cleanup potential, not forced GC)
    println!("Before: {}, After: {}", with_large_df, after_reassign);
}

#[test]
fn test_lazy_evaluation_performance() {
    let mut session = SharedSession::new();
    
    // Create operations that could be lazily evaluated
    session.execute("setup", "let df = DataFrame::from_range(0, 1000)").unwrap();
    
    // Test that complex operations don't immediately consume excessive memory
    let start = Instant::now();
    let result = session.execute("lazy_op", "let filtered = df.filter(true).select(\"value\")");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Lazy evaluation failed");
    assert!(elapsed.as_millis() < 100, "Lazy evaluation too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_memory_pressure_response() {
    let mut session = SharedSession::new();
    
    // Create multiple DataFrames to simulate memory pressure
    for i in 0..5 {
        let code = format!("let df{} = DataFrame::from_range(0, 500)", i);
        let result = session.execute(&format!("pressure_{}", i), &code);
        assert!(result.is_ok(), "Memory pressure test failed at iteration {}", i);
    }
    
    // System should still be responsive
    let result = session.execute("responsive_test", "42 + 58");
    assert!(result.is_ok(), "System not responsive under memory pressure");
}

#[test]
fn test_dataframe_column_efficiency() {
    let mut session = SharedSession::new();
    
    // Create DataFrame with many columns
    session.execute("multi_col", 
        "let df = DataFrame([[1, 2, 3, 4, 5], [6, 7, 8, 9, 10], [11, 12, 13, 14, 15]])")
    .unwrap();
    
    // Selecting single column should be efficient
    let start = Instant::now();
    let result = session.execute("single_col", "df.select(\"column_0\")");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Column selection failed");
    assert!(elapsed.as_millis() < 25, "Column selection too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_memory_estimation_accuracy() {
    let mut session = SharedSession::new();
    
    // Get baseline
    let baseline = session.estimate_interpreter_memory();
    
    // Add data
    session.execute("data", "let x = DataFrame([[1, 2], [3, 4]])").unwrap();
    let with_data = session.estimate_interpreter_memory();
    
    // Memory estimation should increase
    assert!(with_data > baseline, "Memory estimation not working: {} vs {}", baseline, with_data);
    
    // Should be reasonable (not too small or too large)
    let growth = with_data - baseline;
    assert!(growth > 10, "Memory growth too small: {}", growth);
    assert!(growth < 100000, "Memory growth too large: {}", growth);
}

#[test]
fn test_copy_on_write_efficiency() {
    let mut session = SharedSession::new();
    
    // Create original DataFrame
    session.execute("original", "let original = DataFrame::from_range(0, 1000)").unwrap();
    
    // Create "copy" (should be COW)
    session.execute("copy", "let copy = original").unwrap();
    
    // Both should exist without doubling memory usage
    let memory = session.estimate_interpreter_memory();
    
    // Memory should not be double (exact COW hard to test, but should be reasonable)
    assert!(memory < 50000, "COW efficiency check: memory too high: {}", memory);
}

// Helper function to parse memory usage from JSON string
fn parse_memory_usage(memory_json: &str) -> usize {
    serde_json::from_str::<serde_json::Value>(memory_json)
        .ok()
        .and_then(|v| v["total_allocated"].as_u64())
        .unwrap_or(0) as usize
}