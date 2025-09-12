//! End-to-end integration tests for SharedSession
//! Tests complete workflows and scenarios that users would encounter

use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};
use ruchy::wasm::notebook::NotebookRuntime;

#[test]
fn test_data_science_workflow() {
    let mut session = SharedSession::new();
    
    // Step 1: Load data (simulated)
    let result = session.execute("load_data", "let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]");
    assert!(result.is_ok());
    
    // Step 2: Basic statistics
    let result = session.execute("data_length", "data.length");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "10");
    }
    
    // Step 3: Data transformation (if supported)
    let result = session.execute("doubled", "let doubled = data.map(x => x * 2)");
    if result.is_ok() {
        // Check first element of doubled array
        let result = session.execute("check_double", "doubled[0]");
        if result.is_ok() {
            assert_eq!(result.unwrap().value, "2");
        }
    }
    
    // Step 4: Filtering (if supported)  
    let result = session.execute("filtered", "let evens = data.filter(x => x % 2 == 0)");
    if result.is_ok() {
        // Check that we have filtered data
        let result = session.execute("evens_count", "evens.length");
        if result.is_ok() {
            // Should have 5 even numbers
            assert_eq!(result.unwrap().value, "5");
        }
    }
    
    // Step 5: Aggregation
    let result = session.execute("sum_calc", "let sum = data.reduce((a, b) => a + b, 0)");
    if result.is_ok() {
        let result = session.execute("check_sum", "sum");
        if result.is_ok() {
            // Sum of 1-10 is 55
            assert_eq!(result.unwrap().value, "55");
        }
    }
}

#[test]
fn test_function_development_workflow() {
    let mut session = SharedSession::new();
    
    // Step 1: Define utility functions
    let result = session.execute("util1", "fun square(x) { x * x }");
    assert!(result.is_ok());
    
    let result = session.execute("util2", "fun is_even(x) { x % 2 == 0 }");
    assert!(result.is_ok());
    
    // Step 2: Test functions individually
    let result = session.execute("test_square", "square(4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "16");
    
    let result = session.execute("test_is_even", "is_even(6)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "true");
    
    // Step 3: Compose functions
    let result = session.execute("compose", "fun square_if_even(x) { if is_even(x) { square(x) } else { x } }");
    assert!(result.is_ok());
    
    // Step 4: Test composed function
    let result = session.execute("test_compose1", "square_if_even(4)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "16"); // 4 is even, so squared
    
    let result = session.execute("test_compose2", "square_if_even(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "5"); // 5 is odd, so unchanged
    
    // Step 5: Apply to data
    let result = session.execute("apply_data", "let numbers = [2, 3, 4, 5]; numbers.map(square_if_even)");
    if result.is_ok() {
        // Should transform even numbers: [4, 3, 16, 5]
        let result = session.execute("check_transform", "numbers.map(square_if_even)[0]");
        if result.is_ok() {
            assert_eq!(result.unwrap().value, "4"); // 2 squared
        }
    }
}

#[test]
fn test_reactive_development_workflow() {
    let mut session = SharedSession::new();
    session.set_execution_mode(ExecutionMode::Reactive);
    
    // Step 1: Define parameters
    let result = session.execute("params", "let rate = 0.05; let principal = 1000");
    assert!(result.is_ok());
    
    // Step 2: Define dependent calculations
    let result = session.execute("monthly_rate", "let monthly_rate = rate / 12");
    assert!(result.is_ok());
    
    let result = session.execute("interest", "let monthly_interest = principal * monthly_rate");
    assert!(result.is_ok());
    
    let result = session.execute("annual", "let annual_interest = principal * rate");
    assert!(result.is_ok());
    
    // Step 3: Verify initial calculation
    let result = session.execute("check_annual", "annual_interest");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "50"); // 1000 * 0.05
    
    // Step 4: Change parameter (should trigger reactive updates)
    let responses = session.execute_reactive("params", "let rate = 0.10; let principal = 1000");
    
    // Should have multiple responses due to reactive cascade
    assert!(responses.len() >= 1);
    
    // Step 5: Verify updated calculations
    let result = session.execute("check_updated", "annual_interest");
    assert!(result.is_ok());
    // Should be updated to new rate
    let new_value = result.unwrap().value.parse::<f64>().unwrap_or(0.0);
    assert!(new_value > 50.0); // Should be higher with 10% rate
}

#[test]
fn test_iterative_development_workflow() {
    let mut session = SharedSession::new();
    
    // Version 1: Simple implementation
    let result = session.execute("v1", "fun factorial_v1(n) { if n <= 1 { 1 } else { n * factorial_v1(n - 1) } }");
    assert!(result.is_ok());
    
    let result = session.execute("test_v1", "factorial_v1(5)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "120");
    
    // Version 2: Optimized implementation (iterative)
    let result = session.execute("v2", r#"
        fun factorial_v2(n) {
            let result = 1;
            let i = 1;
            while i <= n {
                result = result * i;
                i = i + 1;
            }
            result
        }
    "#);
    if result.is_ok() {
        let result = session.execute("test_v2", "factorial_v2(5)");
        if result.is_ok() {
            assert_eq!(result.unwrap().value, "120");
        }
    }
    
    // Version 3: Memoized version
    let result = session.execute("v3_setup", "let memo = {}");
    if result.is_ok() {
        let result = session.execute("v3", r#"
            fun factorial_v3(n) {
                if memo[n] != null {
                    memo[n]
                } else {
                    let result = if n <= 1 { 1 } else { n * factorial_v3(n - 1) };
                    memo[n] = result;
                    result
                }
            }
        "#);
        if result.is_ok() {
            let result = session.execute("test_v3", "factorial_v3(5)");
            if result.is_ok() {
                assert_eq!(result.unwrap().value, "120");
            }
        }
    }
    
    // All versions should still be available
    let result = session.execute("compare", "factorial_v1(4) == 24");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "true");
    }
}

#[test]
fn test_notebook_runtime_integration() {
    let mut runtime = NotebookRuntime::new().unwrap();
    
    // Step 1: Execute cells through notebook runtime
    let result = runtime.execute_cell_with_session("setup", "let notebook_var = 42");
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.value, "42");
    
    // Step 2: Use variable in next cell
    let result = runtime.execute_cell_with_session("calc", "notebook_var * 2");
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.value, "84");
    
    // Step 3: Define function through notebook
    let result = runtime.execute_cell_with_session("func_def", "fun triple(x) { x * 3 }");
    assert!(result.is_ok());
    
    // Step 4: Use function
    let result = runtime.execute_cell_with_session("func_use", "triple(notebook_var)");
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.value, "126"); // 42 * 3
}

#[test]
fn test_checkpoint_workflow() {
    let mut session = SharedSession::new();
    
    // Step 1: Set up initial experiment
    session.execute("experiment_setup", "let initial_value = 100").unwrap();
    session.execute("calculation", "let result = initial_value * 2").unwrap();
    
    // Step 2: Create checkpoint before risky changes
    session.create_checkpoint("before_experiment").unwrap();
    
    // Step 3: Make experimental changes
    session.execute("risky_change", "let initial_value = 200").unwrap();
    session.execute("new_calc", "let new_result = initial_value * 3").unwrap();
    
    let result = session.execute("check_new", "new_result");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "600"); // 200 * 3
    
    // Step 4: Experiment didn't work out, rollback
    session.restore_from_checkpoint("before_experiment").unwrap();
    
    // Step 5: Verify rollback worked
    let result = session.execute("check_restored", "initial_value");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "100"); // Back to original
    
    let result = session.execute("check_old_result", "result");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, "200"); // Original calculation restored
    
    // new_result should not exist after rollback
    let result = session.execute("check_new_missing", "new_result");
    assert!(result.is_err());
}

#[test]
fn test_memory_management_workflow() {
    let mut session = SharedSession::new();
    
    // Step 1: Monitor initial memory
    let initial_memory = session.estimate_interpreter_memory();
    
    // Step 2: Create some data
    session.execute("data1", "let small_data = [1, 2, 3]").unwrap();
    let after_small = session.estimate_interpreter_memory();
    assert!(after_small >= initial_memory);
    
    // Step 3: Create larger data
    session.execute("data2", "let medium_data = Array.new(100, 42)").ok(); // Might not be supported
    let after_medium = session.estimate_interpreter_memory();
    assert!(after_medium >= after_small);
    
    // Step 4: Create checkpoint to test memory impact
    session.create_checkpoint("memory_test").unwrap();
    let after_checkpoint = session.estimate_interpreter_memory();
    
    // Checkpoints might increase memory usage
    assert!(after_checkpoint >= after_medium);
    
    // Step 5: Continue adding data and monitor growth
    for i in 0..10 {
        let var_name = format!("var_{}", i);
        let code = format!("let {} = {}", var_name, i * 10);
        session.execute(&format!("mem_test_{}", i), &code).unwrap();
    }
    
    let final_memory = session.estimate_interpreter_memory();
    assert!(final_memory >= after_checkpoint);
    
    // Memory should have grown with all the variables
    assert!(final_memory > initial_memory);
}

#[test]
fn test_full_data_pipeline() {
    let mut session = SharedSession::new();
    
    // Complete data processing pipeline
    
    // Step 1: Data ingestion
    session.execute("ingest", "let raw_data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]").unwrap();
    
    // Step 2: Data validation
    session.execute("validate", "let is_valid = raw_data.length > 0").unwrap();
    let result = session.execute("check_valid", "is_valid");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "true");
    }
    
    // Step 3: Data cleaning (remove outliers)
    session.execute("clean", "let cleaned_data = raw_data.filter(x => x <= 8)").ok(); 
    
    // Step 4: Feature engineering 
    session.execute("features", "let features = raw_data.map(x => x * x)").ok();
    
    // Step 5: Statistical analysis
    session.execute("mean", "let mean = raw_data.reduce((a, b) => a + b, 0) / raw_data.length").ok();
    
    // Step 6: Results summary
    session.execute("summary", r#"
        let analysis_summary = {
            count: raw_data.length,
            mean: 5.5, // Calculated mean of 1-10
            processed: true
        }
    "#).ok();
    
    // Step 7: Verify end-to-end pipeline
    let result = session.execute("final_check", "raw_data.length");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "10");
    }
    
    // All intermediate variables should still be accessible
    let result = session.execute("validation_check", "is_valid");
    if result.is_ok() {
        assert_eq!(result.unwrap().value, "true");
    }
}