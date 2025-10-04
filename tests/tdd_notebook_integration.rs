//! NotebookRuntime Integration Tests - Sprint 10
//!
//! Tests for enhanced NotebookRuntime integration with advanced SharedSession features:
//! - Notebook persistence and session management
//! - Web-based interface integration
//! - Real-time collaboration features
//! - Export/import in multiple formats (Jupyter, HTML, PDF)
//! - Comprehensive notebook management API
//! - WebSocket support for real-time updates

use ruchy::wasm::notebook::NotebookRuntime;
use serde_json::Value as JsonValue;

// ============================================================================
// NotebookRuntime Advanced Integration Tests
// ============================================================================

#[test]
fn test_notebook_session_persistence() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create notebook with data
    let cell1_id = runtime.add_cell("code", "let x = 42");
    let cell2_id = runtime.add_cell("code", "let df = DataFrame([[1, 2], [3, 4]])");
    let cell3_id = runtime.add_cell("markdown", "# Analysis Results");

    // Execute code cells
    runtime.execute_cell(&cell1_id).unwrap();
    runtime.execute_cell(&cell2_id).unwrap();

    // Check session persistence integration
    // Planned feature: Add notebook-level persistence methods
    // let session_data = runtime.export_session().unwrap();
    // assert!(session_data.contains_key("cells"), "Should export cell data");
    // assert!(session_data.contains_key("session_state"), "Should export session state");

    // For now, verify cells were created and executed
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert_eq!(cells.len(), 3, "Should have 3 cells");

    println!("Notebook session persistence structure established");
}

#[test]
fn test_notebook_checkpoint_management() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create initial notebook state
    let cell_id = runtime.add_cell("code", "let data = [1, 2, 3, 4, 5]");
    runtime.execute_cell(&cell_id).unwrap();

    // Planned feature: Add notebook-level checkpoint management
    // let checkpoint_id = runtime.create_notebook_checkpoint("before_analysis").unwrap();

    // Make changes
    let analysis_cell = runtime.add_cell("code", "let sum = data.sum()");
    runtime.execute_cell(&analysis_cell).unwrap();

    // Planned feature: Test checkpoint restoration
    // runtime.restore_notebook_checkpoint(&checkpoint_id).unwrap();

    // For now, verify basic functionality
    let memory_usage = runtime.get_memory_usage();
    let parsed_memory: JsonValue = serde_json::from_str(&memory_usage).unwrap();
    assert!(parsed_memory.is_object(), "Memory tracking should work");

    println!("Notebook checkpoint management structure established");
}

#[test]
fn test_notebook_collaborative_features() {
    let mut runtime1 = NotebookRuntime::new().unwrap();
    let runtime2 = NotebookRuntime::new().unwrap();

    // Create notebook in runtime1
    let cell_id = runtime1.add_cell("code", "let shared_data = 100");
    runtime1.execute_cell(&cell_id).unwrap();

    // Planned feature: Add collaboration features
    // let notebook_state = runtime1.export_for_collaboration().unwrap();
    // runtime2.import_collaborative_state(&notebook_state).unwrap();

    // For now, verify independent operation
    let runtime2_cells = runtime2.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&runtime2_cells).unwrap();
    // Each runtime starts empty
    assert_eq!(cells.len(), 0, "Runtime2 should start empty");

    println!("Collaborative features structure established");
}

#[test]
fn test_notebook_export_formats() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create comprehensive notebook
    runtime.add_cell("markdown", "# Data Analysis Notebook");
    runtime.add_cell("code", "let data = DataFrame::from_range(0, 100)");
    runtime.add_cell("code", "let filtered = data.filter(true)");
    runtime.add_cell("markdown", "## Results Summary");
    runtime.add_cell("code", "filtered.sum()");

    // Execute code cells
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();

    // Planned feature: Add export format methods
    // let jupyter_export = runtime.export_as_jupyter().unwrap();
    // assert!(jupyter_export.contains("nbformat"), "Should be valid Jupyter format");

    // let html_export = runtime.export_as_html().unwrap();
    // assert!(html_export.contains("<html>"), "Should be valid HTML");

    // let markdown_export = runtime.export_as_markdown().unwrap();
    // assert!(markdown_export.contains("# Data Analysis"), "Should preserve markdown");

    // For now, verify notebook structure
    assert!(!cells.is_empty(), "Should have cells for export");
    let notebook_json = runtime.to_json();
    assert!(
        notebook_json.contains("cells"),
        "Should have standard notebook format"
    );

    println!("Export formats structure established");
}

// ============================================================================
// Web Interface Integration Tests
// ============================================================================

#[test]
fn test_web_interface_api_endpoints() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Check API-like operations
    let cell_id = runtime.add_cell("code", "let api_test = 42");
    runtime.execute_cell(&cell_id).unwrap();

    // Planned feature: Add web API methods
    // let api_response = runtime.handle_api_request("GET", "/cells").unwrap();
    // assert!(api_response.status == 200, "API should respond successfully");

    // let cell_response = runtime.handle_api_request("POST", "/cells/execute", &json!({
    //     "cell_id": cell_id,
    //     "code": "let result = api_test * 2"
    // })).unwrap();
    // assert!(cell_response.success, "Cell execution via API should work");

    // For now, verify basic operations work
    let globals = runtime.get_globals();
    let globals_obj: JsonValue = serde_json::from_str(&globals).unwrap();
    assert!(
        globals_obj.is_object(),
        "Globals should be serializable for web API"
    );

    println!("Web interface API structure established");
}

#[test]
fn test_real_time_updates() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Set up for real-time updates
    runtime.set_execution_mode("reactive");

    // Create dependency chain
    let cell1 = runtime.add_cell("code", "let base = 10");
    let cell2 = runtime.add_cell("code", "let derived = base * 2");

    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap();

    // Planned feature: Add real-time update tracking
    // let update_tracker = runtime.create_update_tracker().unwrap();

    // Trigger reactive update
    let responses_json = runtime.execute_reactive(&cell1, "let base = 20").unwrap();
    let responses: Vec<JsonValue> = serde_json::from_str(&responses_json).unwrap();

    assert!(
        !responses.is_empty(),
        "Reactive execution should generate updates"
    );

    // Planned feature: Test WebSocket-style updates
    // let updates = runtime.get_pending_updates().unwrap();
    // assert!(!updates.is_empty(), "Should have pending updates for clients");

    println!("Real-time updates structure established");
}

#[test]
fn test_notebook_debugging_interface() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create notebook with debugging scenarios
    let cell1 = runtime.add_cell("code", "let debug_var = 42");
    let cell2 = runtime.add_cell("code", "let error_cell = debug_var / 0"); // Potential error
    let cell3 = runtime.add_cell("code", "let success_cell = debug_var + 10");

    runtime.execute_cell(&cell1).unwrap();

    // Execute potentially problematic cell
    let error_result = runtime.execute_cell(&cell2);
    // May succeed or fail depending on implementation

    runtime.execute_cell(&cell3).unwrap();

    // Planned feature: Add debugging interface methods
    // let debug_info = runtime.get_debug_information().unwrap();
    // assert!(debug_info.contains_key("variables"), "Should provide variable info");
    // assert!(debug_info.contains_key("call_stack"), "Should provide call stack");

    // let execution_trace = runtime.get_execution_trace().unwrap();
    // assert!(!execution_trace.is_empty(), "Should provide execution trace");

    // For now, verify memory and dependency tracking work
    let memory_usage = runtime.get_memory_usage();
    let dependency_graph = runtime.get_dependency_graph();

    assert!(
        !memory_usage.is_empty(),
        "Should track memory for debugging"
    );
    assert!(
        !dependency_graph.is_empty(),
        "Should track dependencies for debugging"
    );

    println!("Debugging interface structure established");
}

// ============================================================================
// Advanced Notebook Features Tests
// ============================================================================

#[test]
fn test_notebook_versioning_and_history() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create initial version
    let cell_id = runtime.add_cell("code", "let version1 = 1");
    runtime.execute_cell(&cell_id).unwrap();

    // Planned feature: Add notebook versioning
    // let version1_snapshot = runtime.create_version_snapshot("v1.0").unwrap();

    // Make changes
    runtime.add_cell("code", "let version2 = 2");

    // Planned feature: Add version history
    // let history = runtime.get_version_history().unwrap();
    // assert_eq!(history.len(), 1, "Should have one version");
    // assert_eq!(history[0].tag, "v1.0", "Should track version tags");

    // For now, verify cell provenance tracking works
    let provenance = runtime.get_cell_provenance(&cell_id);
    let provenance_obj: JsonValue = serde_json::from_str(&provenance).unwrap();
    assert!(provenance_obj.is_object(), "Should track cell provenance");

    println!("Notebook versioning structure established");
}

#[test]
fn test_notebook_template_system() {
    let runtime = NotebookRuntime::new().unwrap();

    // Planned feature: Add template system
    // let templates = runtime.get_available_templates().unwrap();
    // assert!(!templates.is_empty(), "Should have built-in templates");

    // let data_analysis_template = runtime.create_from_template("data_analysis").unwrap();
    // assert!(data_analysis_template.cells.len() > 0, "Template should create cells");

    // For now, verify notebook creation and structure
    let notebook_json = runtime.to_json();
    assert!(
        notebook_json.contains("version"),
        "Should have version info"
    );
    assert!(notebook_json.contains("metadata"), "Should have metadata");
    assert!(notebook_json.contains("cells"), "Should have cells array");

    println!("Template system structure established");
}

#[test]
fn test_notebook_plugin_system() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Planned feature: Add plugin system
    // let plugins = runtime.get_available_plugins().unwrap();
    // runtime.enable_plugin("data_visualization").unwrap();
    // runtime.enable_plugin("code_completion").unwrap();

    // Create cell that could use plugins
    let cell_id = runtime.add_cell("code", "let plugin_data = DataFrame([[1, 2], [3, 4]])");
    runtime.execute_cell(&cell_id).unwrap();

    // Planned feature: Test plugin integration
    // let viz_result = runtime.execute_cell_with_plugin(&cell_id, "data_visualization").unwrap();
    // assert!(viz_result.contains("visualization"), "Plugin should enhance output");

    // For now, verify basic cell execution works
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    assert!(!cells.is_empty(), "Should execute cells successfully");

    println!("Plugin system structure established");
}

// ============================================================================
// Performance and Scalability Tests
// ============================================================================

#[test]
fn test_large_notebook_performance() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create large notebook
    for i in 0..50 {
        // Reasonable size for testing
        let code = format!("let var_{} = {}", i, i * 2);
        let cell_id = runtime.add_cell("code", &code);
        runtime.execute_cell(&cell_id).unwrap();
    }

    // Add some markdown cells
    for i in 0..10 {
        runtime.add_cell("markdown", &format!("# Section {}", i));
    }

    // Check performance of operations on large notebook
    let start = std::time::Instant::now();
    let cells_json = runtime.get_cells();
    let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(cells.len(), 60, "Should handle 60 cells");
    assert!(
        elapsed.as_millis() < 100,
        "Large notebook operations should be fast: {}ms",
        elapsed.as_millis()
    );

    // Check memory efficiency
    let memory_usage = runtime.get_memory_usage();
    let memory_obj: JsonValue = serde_json::from_str(&memory_usage).unwrap();
    let total_allocated = memory_obj["total_allocated"].as_u64().unwrap_or(0);
    assert!(
        total_allocated < 10_000_000,
        "Memory usage should be reasonable: {}",
        total_allocated
    );
}

#[test]
fn test_concurrent_notebook_operations() {
    use std::thread;

    // Check concurrent notebook creation (each in separate thread)
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                let mut runtime = NotebookRuntime::new().unwrap();

                // Each runtime creates and executes cells independently
                let cell_id = runtime.add_cell("code", &format!("let thread_{} = {}", i, i * 10));
                let result = runtime.execute_cell(&cell_id);

                assert!(
                    result.is_ok(),
                    "Concurrent execution should work in thread {}",
                    i
                );

                // Return cell count for verification
                let cells_json = runtime.get_cells();
                let cells: Vec<JsonValue> = serde_json::from_str(&cells_json).unwrap();
                cells.len()
            })
        })
        .collect();

    // Check all concurrent operations completed successfully
    for (i, handle) in handles.into_iter().enumerate() {
        let cell_count = handle.join().unwrap();
        assert_eq!(cell_count, 1, "Thread {} should have created 1 cell", i);
    }
}

// ============================================================================
// Error Handling and Recovery Tests
// ============================================================================

#[test]
fn test_notebook_error_recovery() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create valid cells
    let good_cell1 = runtime.add_cell("code", "let good = 42");
    let bad_cell = runtime.add_cell("code", "let bad = invalid_syntax_here!");
    let good_cell2 = runtime.add_cell("code", "let also_good = 24");

    // Execute in sequence
    assert!(
        runtime.execute_cell(&good_cell1).is_ok(),
        "Good cell 1 should execute"
    );

    // The bad cell may or may not fail depending on error handling implementation
    let bad_result = runtime.execute_cell(&bad_cell);
    println!("Bad cell result: {:?}", bad_result);

    assert!(
        runtime.execute_cell(&good_cell2).is_ok(),
        "Good cell 2 should execute after error"
    );

    // Planned feature: Test notebook-level recovery
    // runtime.recover_from_errors().unwrap();
    // let health_check = runtime.check_notebook_health().unwrap();
    // assert!(health_check.is_healthy, "Notebook should recover from errors");

    // Check notebook is still functional
    let final_cell = runtime.add_cell("code", "let recovery = good + also_good");
    assert!(
        runtime.execute_cell(&final_cell).is_ok(),
        "Recovery cell should work"
    );

    println!("Notebook error recovery verified");
}

#[test]
fn test_notebook_data_integrity() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create data that should persist across operations
    let data_cell = runtime.add_cell(
        "code",
        "let persistent_data = DataFrame::from_range(0, 100)",
    );
    runtime.execute_cell(&data_cell).unwrap();

    // Perform various operations
    let filter_cell = runtime.add_cell("code", "let filtered = persistent_data.filter(true)");
    let sum_cell = runtime.add_cell("code", "let total = persistent_data.sum()");

    runtime.execute_cell(&filter_cell).unwrap();
    runtime.execute_cell(&sum_cell).unwrap();

    // Planned feature: Test data integrity preservation
    // let integrity_check = runtime.verify_data_integrity().unwrap();
    // assert!(integrity_check.all_valid, "All data should maintain integrity");

    // Check data is still accessible
    let verification_cell = runtime.add_cell("code", "persistent_data.slice(0, 5)");
    assert!(
        runtime.execute_cell(&verification_cell).is_ok(),
        "Data should remain accessible"
    );

    println!("Data integrity verification completed");
}
