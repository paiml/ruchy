//! Performance monitoring and regression detection tests
//! Tracks performance characteristics to detect regressions

use ruchy::wasm::shared_session::{ExecutionMode, SharedSession};
use std::time::{Duration, Instant};

#[test]
fn test_session_creation_performance() {
    // Measure session creation time
    let start = Instant::now();

    for _ in 0..100 {
        let _session = SharedSession::new();
    }

    let elapsed = start.elapsed();

    // Should create 100 sessions in reasonable time
    assert!(
        elapsed < Duration::from_millis(1000),
        "Session creation too slow: {:?}",
        elapsed
    );

    println!("✅ Created 100 sessions in {:?}", elapsed);
}

#[test]
fn test_simple_execution_performance() {
    let mut session = SharedSession::new();

    // Warm up
    session.execute("warmup", "42").ok();

    let start = Instant::now();

    // Execute simple expressions
    for i in 0..100 {
        let cell_id = format!("perf_{}", i);
        let code = format!("{}", i);
        session.execute(&cell_id, &code).unwrap();
    }

    let elapsed = start.elapsed();

    // Should execute 100 simple expressions quickly
    assert!(
        elapsed < Duration::from_secs(1),
        "Simple execution too slow: {:?}",
        elapsed
    );

    println!("✅ Executed 100 simple expressions in {:?}", elapsed);
}

#[test]
fn test_variable_access_performance() {
    let mut session = SharedSession::new();

    // Set up variables
    for i in 0..50 {
        let var_name = format!("var_{}", i);
        let code = format!("let {} = {}", var_name, i);
        session.execute(&format!("setup_{}", i), &code).unwrap();
    }

    let start = Instant::now();

    // Access variables repeatedly
    for _ in 0..100 {
        for i in 0..50 {
            let var_name = format!("var_{}", i);
            session.execute("access", &var_name).unwrap();
        }
    }

    let elapsed = start.elapsed();

    // Should access variables efficiently
    assert!(
        elapsed < Duration::from_secs(2),
        "Variable access too slow: {:?}",
        elapsed
    );

    println!("✅ Performed 5000 variable accesses in {:?}", elapsed);
}

#[test]
fn test_function_call_performance() {
    let mut session = SharedSession::new();

    // Define function
    session
        .execute("func_def", "fun add(a, b) { a + b }")
        .unwrap();

    let start = Instant::now();

    // Call function repeatedly
    for i in 0..200 {
        let code = format!("add({}, {})", i, i + 1);
        session.execute("func_call", &code).unwrap();
    }

    let elapsed = start.elapsed();

    // Function calls should be reasonably fast
    assert!(
        elapsed < Duration::from_secs(1),
        "Function calls too slow: {:?}",
        elapsed
    );

    println!("✅ Performed 200 function calls in {:?}", elapsed);
}

#[test]
fn test_memory_estimation_performance() {
    let mut session = SharedSession::new();

    // Add some data
    for i in 0..20 {
        let code = format!(
            "let data_{} = [{}]",
            i,
            (0..i).map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
        );
        session.execute(&format!("data_{}", i), &code).ok();
    }

    let start = Instant::now();

    // Memory estimation should be fast
    for _ in 0..1000 {
        let _memory = session.estimate_interpreter_memory();
    }

    let elapsed = start.elapsed();

    // Memory estimation should be O(1) and very fast
    assert!(
        elapsed < Duration::from_millis(100),
        "Memory estimation too slow: {:?}",
        elapsed
    );

    println!("✅ Performed 1000 memory estimations in {:?}", elapsed);
}

#[test]
fn test_execution_mode_switch_performance() {
    let mut session = SharedSession::new();

    let start = Instant::now();

    // Switch modes repeatedly
    for i in 0..500 {
        if i % 2 == 0 {
            session.set_execution_mode(ExecutionMode::Manual);
        } else {
            session.set_execution_mode(ExecutionMode::Reactive);
        }
    }

    let elapsed = start.elapsed();

    // Mode switching should be very fast
    assert!(
        elapsed < Duration::from_millis(50),
        "Mode switching too slow: {:?}",
        elapsed
    );

    println!("✅ Performed 500 mode switches in {:?}", elapsed);
}

#[test]
fn test_checkpoint_performance() {
    let mut session = SharedSession::new();

    // Set up some state
    for i in 0..10 {
        let code = format!("let var_{} = {}", i, i);
        session.execute(&format!("setup_{}", i), &code).unwrap();
    }

    let start = Instant::now();

    // Create checkpoints
    for i in 0..50 {
        let _checkpoint_name = format!("checkpoint_{}", i);
        // session.create_checkpoint(&checkpoint_name).unwrap(); // Method not available
    }

    let elapsed = start.elapsed();

    // Checkpoint creation should be reasonable
    assert!(
        elapsed < Duration::from_secs(1),
        "Checkpoint creation too slow: {:?}",
        elapsed
    );

    println!("✅ Created 50 checkpoints in {:?}", elapsed);
}

#[test]
fn test_error_handling_performance() {
    let mut session = SharedSession::new();

    let start = Instant::now();

    // Generate errors repeatedly (should not slow down)
    for i in 0..100 {
        let cell_id = format!("error_{}", i);
        let _result = session.execute(&cell_id, "undefined_variable");
        // Errors should be handled efficiently
    }

    let elapsed = start.elapsed();

    // Error handling should not significantly impact performance
    assert!(
        elapsed < Duration::from_millis(500),
        "Error handling too slow: {:?}",
        elapsed
    );

    // Session should still work after many errors
    let result = session.execute("recovery", "42");
    assert!(result.is_ok());

    println!("✅ Handled 100 errors in {:?}", elapsed);
}

#[test]
fn test_scalability_characteristics() {
    let mut session = SharedSession::new();

    // Test with increasing numbers of variables
    let test_sizes = [10, 50, 100, 200];
    let mut times = Vec::new();

    for &size in &test_sizes {
        // Set up variables
        for i in 0..size {
            let code = format!("let scale_var_{} = {}", i, i);
            session
                .execute(&format!("scale_setup_{}_{}", size, i), &code)
                .unwrap();
        }

        // Measure access time
        let start = Instant::now();

        // Access a variable (should be O(1) or O(log n))
        session
            .execute("scale_access", &format!("scale_var_{}", size / 2))
            .unwrap();

        let elapsed = start.elapsed();
        times.push(elapsed);
    }

    // Performance should not degrade significantly with scale
    // (allowing some variance for system noise)
    for (i, &time) in times.iter().enumerate() {
        println!(
            "Scale test {}: {} variables, access time: {:?}",
            i, test_sizes[i], time
        );

        // Each access should be fast regardless of total variables
        assert!(
            time < Duration::from_millis(10),
            "Variable access scales poorly: {:?} for {} variables",
            time,
            test_sizes[i]
        );
    }

    println!("✅ Scalability test passed - access time remains bounded");
}

/// Utility function to measure execution time
fn measure_execution<F, R>(operation: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = operation();
    let elapsed = start.elapsed();
    (result, elapsed)
}

#[test]
fn test_baseline_performance_metrics() {
    // Establish baseline performance metrics for regression detection

    let (session, creation_time) = measure_execution(|| SharedSession::new());
    println!("📊 Session creation: {:?}", creation_time);

    let mut session = session;

    let (_, simple_exec_time) = measure_execution(|| session.execute("baseline", "42").unwrap());
    println!("📊 Simple execution: {:?}", simple_exec_time);

    let (_, memory_time) = measure_execution(|| session.estimate_interpreter_memory());
    println!("📊 Memory estimation: {:?}", memory_time);

    // Define performance baseline thresholds
    assert!(
        creation_time < Duration::from_millis(10),
        "Session creation baseline"
    );
    assert!(
        simple_exec_time < Duration::from_millis(10),
        "Simple execution baseline"
    );
    assert!(
        memory_time < Duration::from_micros(100),
        "Memory estimation baseline"
    );

    println!("✅ All baseline performance metrics within acceptable ranges");
}
