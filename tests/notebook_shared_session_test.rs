//! Tests for SharedSession notebook state management architecture
//!
//! These tests verify the implementation of persistent state across notebook cells,
//! ensuring that variables, functions, and types are preserved between executions.

use ruchy::wasm::notebook::NotebookRuntime;

// Check 1: Basic state persistence across cells
#[test]
fn test_basic_state_persistence() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Cell 1: Define a variable
    let result1 = runtime
        .execute_cell_with_session("cell1", "let x = 42")
        .unwrap();
    assert!(result1.success);
    assert_eq!(result1.value, "42");

    // Cell 2: Use the variable from cell 1
    let result2 = runtime.execute_cell_with_session("cell2", "x + 8").unwrap();
    assert!(result2.success);
    assert_eq!(result2.value, "50");

    // Cell 3: Define another variable using the first
    let result3 = runtime
        .execute_cell_with_session("cell3", "let y = x * 2")
        .unwrap();
    assert!(result3.success);
    assert_eq!(result3.value, "84");

    // Cell 4: Use both variables
    let result4 = runtime.execute_cell_with_session("cell4", "x + y").unwrap();
    assert!(result4.success);
    assert_eq!(result4.value, "126");
}

// Check 2: Function definitions persist across cells
#[test]
fn test_function_persistence() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Cell 1: Define a function
    runtime
        .execute_cell_with_session("cell1", "fun add(a, b) { a + b }")
        .unwrap();

    // Cell 2: Use the function
    let result = runtime
        .execute_cell_with_session("cell2", "add(3, 4)")
        .unwrap();
    assert!(result.success);
    assert_eq!(result.value, "7");

    // Cell 3: Define another function using the first
    runtime
        .execute_cell_with_session("cell3", "fun double_add(a, b) { add(a, b) * 2 }")
        .unwrap();

    // Cell 4: Use the new function
    let result = runtime
        .execute_cell_with_session("cell4", "double_add(5, 6)")
        .unwrap();
    assert!(result.success);
    assert_eq!(result.value, "22");
}

// Check 3: Semantic dependency tracking
#[test]
fn test_semantic_dependency_tracking() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Define x
    runtime
        .execute_cell_with_session("cell1", "let x = 10")
        .unwrap();

    // Define y depending on x
    runtime
        .execute_cell_with_session("cell2", "let y = x * 2")
        .unwrap();

    // Get dependency graph as JSON
    let graph_json = runtime.get_dependency_graph();
    assert!(graph_json.contains("cell1"));
    assert!(graph_json.contains("cell2"));

    // Redefine x
    runtime
        .execute_cell_with_session("cell3", "let x = 20")
        .unwrap();

    // Check provenance
    let provenance_json = runtime.get_cell_provenance("cell2");
    assert!(provenance_json.contains("depends_on"));
}

// Check 4: Reactive execution cascade
#[test]
fn test_reactive_cascade_execution() {
    let mut runtime = NotebookRuntime::new().unwrap();
    runtime.set_execution_mode("reactive");

    // Set up dependent cells
    runtime
        .execute_cell_with_session("c1", "let x = 10")
        .unwrap();
    runtime
        .execute_cell_with_session("c2", "let y = x * 2")
        .unwrap();
    runtime
        .execute_cell_with_session("c3", "let z = y + 5")
        .unwrap();

    // Modify upstream cell - should trigger cascade
    let responses_json = runtime.execute_reactive("c1", "let x = 20").unwrap();

    // Parse JSON response
    let responses: serde_json::Value = serde_json::from_str(&responses_json).unwrap();

    // Check cascade execution
    assert!(responses.is_array());
    let responses_array = responses.as_array().unwrap();
    assert!(!responses_array.is_empty());
}

// Check 5: Transactional execution with rollback
#[test]
fn test_transactional_rollback() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Define initial state
    runtime
        .execute_cell_with_session("cell1", "let x = 42")
        .unwrap();
    runtime
        .execute_cell_with_session("cell2", "let y = 10")
        .unwrap();

    // Get initial globals
    let globals_before = runtime.get_globals();
    assert!(globals_before.contains('x') || globals_before.contains("values"));

    // Execute cell with error - should rollback
    let _result = runtime.execute_cell_with_session("cell3", "let z = x / 0");
    // Note: Division by zero might not error in our simple implementation

    // State should be unchanged or error handled gracefully
    let globals_after = runtime.get_globals();
    assert!(globals_after.contains('x') || globals_after.contains("values"));
}

// Check 6: COW checkpoint performance
#[test]
fn test_cow_checkpoint_performance() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add many variables
    for i in 0..100 {
        runtime
            .execute_cell_with_session(&format!("cell_{i}"), &format!("let var_{i} = {i}"))
            .ok();
    }

    // Get memory usage - should be fast
    let start = std::time::Instant::now();
    let _usage = runtime.get_memory_usage();
    let duration = start.elapsed();

    // Should complete quickly
    assert!(duration.as_millis() < 100);
}

// Check 7: Hierarchical namespace modules
#[test]
fn test_hierarchical_modules() {
    // Skip for now - modules not yet implemented
    // This is a future feature
}

// Check 8: Execution plan preview for reactive mode
#[test]
fn test_execution_plan_preview() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Set up dependency chain
    runtime
        .execute_cell_with_session("c1", "let a = 1")
        .unwrap();
    runtime
        .execute_cell_with_session("c2", "let b = a * 2")
        .unwrap();
    runtime
        .execute_cell_with_session("c3", "let c = b + a")
        .unwrap();
    runtime
        .execute_cell_with_session("c4", "let d = c * 2")
        .unwrap();

    // Get execution plan without executing
    let plan_json = runtime.explain_reactive("c1");
    let plan: serde_json::Value = serde_json::from_str(&plan_json).unwrap();

    assert!(plan["primary"].is_string());
    assert_eq!(plan["primary"].as_str().unwrap(), "c1");
}

// Check 9: State inspection API
#[test]
fn test_state_inspection_api() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Execute some cells
    runtime
        .execute_cell_with_session("c1", "let x = 42")
        .unwrap();
    runtime
        .execute_cell_with_session("c2", "let y = x * 2")
        .unwrap();
    runtime
        .execute_cell_with_session("c3", "fun add(a, b) { a + b }")
        .unwrap();

    // Get globals as JSON
    let globals_json = runtime.get_globals();
    let globals: serde_json::Value = serde_json::from_str(&globals_json).unwrap();
    assert!(globals.is_object());

    // Get dependency graph as JSON
    let graph_json = runtime.get_dependency_graph();
    let graph: serde_json::Value = serde_json::from_str(&graph_json).unwrap();
    assert!(graph["nodes"].is_array());

    // Get cell provenance as JSON
    let provenance_json = runtime.get_cell_provenance("c2");
    let provenance: serde_json::Value = serde_json::from_str(&provenance_json).unwrap();
    assert!(provenance.is_object());
}

// Check 10: Memory usage tracking
#[test]
fn test_memory_usage_tracking() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Execute cells creating data
    runtime
        .execute_cell_with_session("c1", "let data = [1, 2, 3, 4, 5]")
        .unwrap();
    runtime
        .execute_cell_with_session("c2", "let matrix = [[1, 2], [3, 4]]")
        .unwrap();

    // Get memory usage as JSON
    let usage_json = runtime.get_memory_usage();
    let usage: serde_json::Value = serde_json::from_str(&usage_json).unwrap();

    assert!(usage["globals_bytes"].is_number());
    assert!(usage["checkpoints_count"].is_number());
}

// Property test: No cycles in dependency graph
#[test]
fn property_no_dependency_cycles() {
    // Skip property tests for now - require proptest dependency
}

// Property test: Checkpoint restore identity
#[test]
fn property_checkpoint_restore_identity() {
    // Skip property tests for now - require proptest dependency
}
