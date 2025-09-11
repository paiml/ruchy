//! Performance validation tests for SharedSession operations
//! 
//! Tests critical performance thresholds using TDD methodology:
//! - Cell execution latency limits
//! - DataFrame operation performance bounds
//! - Memory usage efficiency checks
//! - Reactive execution performance

use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};
use ruchy::wasm::notebook::NotebookRuntime;
use std::time::Instant;

#[test]
fn test_cell_execution_performance() {
    let mut session = SharedSession::new();
    
    // Simple arithmetic should be very fast
    let start = Instant::now();
    let result = session.execute("perf_test", "42 + 58");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Simple arithmetic failed");
    assert!(elapsed.as_millis() < 10, "Simple arithmetic too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_variable_assignment_performance() {
    let mut session = SharedSession::new();
    
    // Variable assignment should be fast
    let start = Instant::now();
    let result = session.execute("perf_test", "let x = 100; x * 2");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Variable assignment failed");
    assert!(elapsed.as_millis() < 15, "Variable assignment too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_dataframe_creation_performance() {
    let mut session = SharedSession::new();
    
    // Small DataFrame creation should be reasonable
    let start = Instant::now();
    let result = session.execute("perf_test", "DataFrame([[1, 2], [3, 4], [5, 6]])");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "DataFrame creation failed");
    assert!(elapsed.as_millis() < 50, "DataFrame creation too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_dataframe_operation_performance() {
    let mut session = SharedSession::new();
    
    // Set up DataFrame
    session.execute("setup", "let df = DataFrame([[1, 2], [3, 4], [5, 6]])").unwrap();
    
    // Test column selection performance
    let start = Instant::now();
    let result = session.execute("perf_test", "df.select(\"column_0\")");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "DataFrame selection failed");
    assert!(elapsed.as_millis() < 25, "DataFrame selection too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_reactive_execution_performance() {
    let mut session = SharedSession::new();
    session.set_execution_mode(ExecutionMode::Reactive);
    
    // Set up simple dependency chain
    session.execute("cell_a", "let a = 10").unwrap();
    session.execute("cell_b", "let b = a * 2").unwrap();
    
    // Test reactive update performance
    let start = Instant::now();
    let responses = session.execute_reactive("cell_a", "let a = 20");
    let elapsed = start.elapsed();
    
    assert!(!responses.is_empty(), "Reactive execution failed");
    assert!(elapsed.as_millis() < 100, "Reactive execution too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_memory_efficiency_tracking() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Get initial memory baseline
    let initial_memory = runtime.get_memory_usage();
    
    // Create moderate DataFrame
    runtime.execute_cell_with_session("mem_test", "let df = DataFrame::from_range(0, 1000)").unwrap();
    
    // Check memory increased (should be detectable)
    let after_memory = runtime.get_memory_usage();
    
    // Memory should have changed (indicates tracking works)
    assert!(initial_memory != after_memory, "Memory usage not tracked");
    
    // Parse JSON to verify structure (basic validation)
    let parsed: serde_json::Value = serde_json::from_str(&after_memory).unwrap();
    assert!(parsed.is_object(), "Memory usage not properly formatted");
}

#[test]
fn test_globals_serialization_performance() {
    let mut session = SharedSession::new();
    
    // Set up some state
    session.execute("setup1", "let x = 42").unwrap();
    session.execute("setup2", "let y = DataFrame([[1, 2], [3, 4]])").unwrap();
    
    // Test globals serialization performance (alternative to checkpoint test)
    let start = Instant::now();
    let globals = session.globals.serialize_for_inspection();
    let elapsed = start.elapsed();
    
    assert!(globals.as_object().map_or(false, |o| !o.is_empty()), "Globals serialization failed");
    assert!(elapsed.as_millis() < 25, "Globals serialization too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_large_dataframe_performance() {
    let mut session = SharedSession::new();
    
    // Test larger DataFrame (but not huge to avoid timeouts)
    let start = Instant::now();
    let result = session.execute("large_test", "DataFrame::from_range(0, 5000)");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Large DataFrame creation failed");
    assert!(elapsed.as_millis() < 200, "Large DataFrame creation too slow: {}ms", elapsed.as_millis());
}

#[test]  
fn test_aggregation_performance() {
    let mut session = SharedSession::new();
    
    // Set up DataFrame for aggregation
    session.execute("setup", "let df = DataFrame::from_range(0, 1000)").unwrap();
    
    // Test sum aggregation performance
    let start = Instant::now();
    let result = session.execute("agg_test", "df.sum()");
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Aggregation failed");
    assert!(elapsed.as_millis() < 100, "Aggregation too slow: {}ms", elapsed.as_millis());
}

#[test]
fn test_simple_filter_performance() {
    let mut session = SharedSession::new();
    
    // Set up simple DataFrame for filtering (using known working syntax)
    session.execute("setup", "let df = DataFrame([[1, 2], [3, 4], [5, 6]])").unwrap();
    
    // Test simple filter performance (just check that filter can be called)
    let start = Instant::now();
    let result = session.execute("filter_test", "df.filter(true)");
    let elapsed = start.elapsed();
    
    // Filter should work (even if with simple boolean)
    assert!(result.is_ok(), "Filter operation failed: {:?}", result);
    assert!(elapsed.as_millis() < 50, "Filter operation too slow: {}ms", elapsed.as_millis());
}