//! Concurrent execution safety tests for SharedSession
//! 
//! Tests concurrent access patterns using TDD methodology.
//! Note: SharedSession contains Rc<T> which is !Send, so we test
//! concurrent patterns with individual sessions per thread instead.

use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};
use ruchy::wasm::notebook::NotebookRuntime;
use std::thread;

#[test]
fn test_independent_session_creation() {
    // Test that multiple sessions can be created concurrently in separate threads
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            
            // Each session executes independently
            let code = format!("let x{} = {}", i, i * 10);
            let result = session.execute(&format!("init_{}", i), &code);
            
            assert!(result.is_ok(), "Thread {} session creation failed", i);
            
            // Verify the session works
            let globals = session.globals.serialize_for_inspection();
            assert!(globals.as_object().map_or(false, |o| !o.is_empty()), 
                   "Thread {} globals empty", i);
            
            i
        })
    }).collect();
    
    // All threads should complete successfully
    for handle in handles {
        let result = handle.join();
        assert!(result.is_ok(), "Thread failed to complete");
    }
}

#[test]
fn test_notebook_runtime_concurrent_creation() {
    // Test concurrent NotebookRuntime creation (which wraps SharedSession)
    let handles: Vec<_> = (0..3).map(|i| {
        thread::spawn(move || {
            let mut runtime = NotebookRuntime::new().unwrap();
            
            // Each runtime processes cells independently
            let code = format!("let data{} = DataFrame([[{}, {}]])", i, i, i + 1);
            let result = runtime.execute_cell_with_session(&format!("cell_{}", i), &code);
            
            assert!(result.is_ok(), "Thread {} runtime failed", i);
            
            // Test memory tracking works
            let memory = runtime.get_memory_usage();
            let parsed: serde_json::Value = serde_json::from_str(&memory).unwrap();
            assert!(parsed.is_object(), "Thread {} memory tracking failed", i);
            
            i
        })
    }).collect();
    
    // Verify all concurrent runtimes work
    for (expected, handle) in (0..).zip(handles) {
        let actual = handle.join().unwrap();
        assert_eq!(actual, expected, "Thread result mismatch");
    }
}

#[test]
fn test_session_isolation_across_threads() {
    // Test that sessions in different threads don't interfere
    let handles: Vec<_> = (0..3).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            
            // Each session executes a computation with different inputs
            let result = session.execute("computation", &format!("{} * 10 + 5", i * 100));
            
            // Verify computation succeeded and is isolated per thread
            assert!(result.is_ok(), "Thread {} computation failed", i);
            
            // Extract the computed result
            let response = result.unwrap();
            let expected_result = format!("{}", i * 100 * 10 + 5);
            
            // Verify the result is correct for this thread
            assert!(response.value.contains(&expected_result), 
                   "Thread {} - wrong result: expected {}, got {}", i, expected_result, response.value);
            
            i * 100 * 10 + 5
        })
    }).collect();
    
    // Each thread should have computed its own isolated result
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results, vec![5, 1005, 2005], "Session isolation failed");
}

#[test]
fn test_reactive_mode_per_thread() {
    // Test reactive mode in separate threads
    let handles: Vec<_> = (0..2).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            session.set_execution_mode(ExecutionMode::Reactive);
            
            // Set up dependency chain
            session.execute("root", &format!("let root = {}", i)).unwrap();
            session.execute("derived", "let derived = root * 10").unwrap();
            
            // Test reactive execution
            let responses = session.execute_reactive("root", &format!("let root = {}", i + 10));
            
            assert!(!responses.is_empty(), "Thread {} reactive execution failed", i);
            responses.len()
        })
    }).collect();
    
    // Both threads should have working reactive execution
    for (i, handle) in handles.into_iter().enumerate() {
        let response_count = handle.join().unwrap();
        assert!(response_count > 0, "Thread {} reactive responses empty", i);
    }
}

#[test]
fn test_dataframe_operations_per_thread() {
    // Test DataFrame operations in concurrent threads
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            
            // Each thread works with different DataFrame sizes
            let size = (i + 1) * 100;
            session.execute("df_create", &format!("let df = DataFrame::from_range(0, {})", size)).unwrap();
            
            // Test different operations per thread
            let operation = match i {
                0 => "df.sum()",
                1 => "df.select(\"value\")",
                2 => "df.slice(0, 10)",
                _ => "df.filter(true)",
            };
            
            let result = session.execute("df_op", operation);
            assert!(result.is_ok(), "Thread {} DataFrame operation failed", i);
            
            // Return memory usage as a consistency check
            session.estimate_interpreter_memory()
        })
    }).collect();
    
    // All DataFrame operations should succeed
    let memories: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    // Memory usage should increase with DataFrame size (rough check)
    assert!(memories[3] >= memories[0], "Memory usage not scaling with DataFrame size");
}

#[test]
fn test_error_handling_isolation() {
    // Test that errors in one thread don't affect other threads
    let handles: Vec<_> = (0..3).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            
            let result = if i == 1 {
                // Middle thread has intentional error
                session.execute("error_test", "this is invalid syntax!")
            } else {
                // Other threads succeed
                session.execute("success_test", &format!("let success{} = {}", i, i * 42))
            };
            
            (i, result.is_ok())
        })
    }).collect();
    
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    // Verify error isolation
    for (thread_id, success) in results {
        if thread_id == 1 {
            assert!(!success, "Error thread should fail");
        } else {
            assert!(success, "Good thread {} should succeed", thread_id);
        }
    }
}

#[test]
fn test_memory_tracking_per_thread() {
    // Test memory tracking works independently per thread
    let handles: Vec<_> = (0..3).map(|i| {
        thread::spawn(move || {
            let mut runtime = NotebookRuntime::new().unwrap();
            
            // Get initial memory
            let initial = runtime.get_memory_usage();
            
            // Create data proportional to thread ID
            let size = (i + 1) * 200;
            runtime.execute_cell_with_session("mem_test", 
                &format!("let data = DataFrame::from_range(0, {})", size)).unwrap();
            
            let after = runtime.get_memory_usage();
            
            // Memory should increase
            assert!(after != initial, "Thread {} - memory not tracked", i);
            
            // Parse memory to verify format
            let parsed: serde_json::Value = serde_json::from_str(&after).unwrap();
            assert!(parsed["total_allocated"].is_number(), "Thread {} - invalid memory format", i);
            
            parsed["total_allocated"].as_u64().unwrap_or(0) as usize
        })
    }).collect();
    
    let memory_usages: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    
    // Larger DataFrames should generally use more memory
    assert!(memory_usages[2] >= memory_usages[0], "Memory scaling inconsistent");
}

#[test]
fn test_checkpoint_creation_per_thread() {
    // Test checkpoint functionality works per thread (smoke test)
    let handles: Vec<_> = (0..2).map(|i| {
        thread::spawn(move || {
            let mut session = SharedSession::new();
            
            // Create some state
            session.execute("state", &format!("let x = {}", i * 50)).unwrap();
            
            // Create COW checkpoint (basic smoke test - just verify it doesn't panic)
            let _snapshot = session.globals.cow_checkpoint();
            
            // Verify session still works after checkpoint
            let result = session.execute("post_checkpoint", "let y = 123");
            assert!(result.is_ok(), "Thread {} - session broken after checkpoint", i);
            
            i
        })
    }).collect();
    
    // Both threads should create checkpoints successfully
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join().unwrap();
        assert_eq!(result, i, "Thread {} checkpoint test failed", i);
    }
}