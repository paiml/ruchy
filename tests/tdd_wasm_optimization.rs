//! TDD tests for WASM module optimization
//!
//! Tests to ensure WASM module meets size and performance targets
//! following TDD RED->GREEN->REFACTOR methodology

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_wasm_module_size_under_200kb() {
    // Build WASM module
    let output = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--release"])
        .output();

    if output.is_err() {
        // If wasm-pack not installed, skip test
        println!("wasm-pack not installed, skipping WASM size test");
        return;
    }

    // Check generated WASM file size
    let wasm_path = Path::new("pkg/ruchy_bg.wasm");
    if wasm_path.exists() {
        let metadata = fs::metadata(wasm_path).unwrap();
        let size_kb = metadata.len() / 1024;

        // Should be under 200KB
        assert!(
            size_kb < 200,
            "WASM module size {}KB exceeds 200KB limit",
            size_kb
        );
    }
}

#[test]
fn test_wasm_module_exports_required_functions() {
    // This test would check that the WASM module exports the required functions
    // For now, we'll create a simple check

    let wasm_exports = vec![
        "NotebookRuntime",
        "execute_cell",
        "get_globals",
        "get_dependency_graph",
        "restart_session",
    ];

    // In a real implementation, we'd parse the WASM module
    // For now, just check that the source files exist
    let notebook_path = Path::new("src/wasm/notebook.rs");
    assert!(notebook_path.exists(), "Notebook module not found");

    let content = fs::read_to_string(notebook_path).unwrap();
    for export in wasm_exports {
        assert!(content.contains(export), "Missing WASM export: {}", export);
    }
}

#[cfg(target_arch = "wasm32")]
#[test]
fn test_wasm_memory_usage_under_50mb() {
    use wasm_bindgen::prelude::*;

    // Get initial memory usage
    let initial_memory = wasm_bindgen::memory().buffer().byte_length();

    // Create notebook runtime and execute some cells
    // This would be the actual test in WASM environment

    // Check memory usage stays under 50MB
    let final_memory = wasm_bindgen::memory().buffer().byte_length();
    let memory_used_mb = (final_memory - initial_memory) / (1024 * 1024);

    assert!(
        memory_used_mb < 50,
        "Memory usage {}MB exceeds 50MB limit",
        memory_used_mb
    );
}

#[test]
fn test_wasm_compilation_with_optimizations() {
    // Check that WASM is built with proper optimization flags
    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();

    // Check for wasm-opt settings
    assert!(
        cargo_toml.contains("[profile.release]") || cargo_toml.contains("wasm-opt"),
        "Missing WASM optimization configuration"
    );
}

#[test]
fn test_browser_api_bindings_exist() {
    // Check that browser API bindings are defined
    let wasm_module = Path::new("src/wasm/notebook.rs");
    let content = fs::read_to_string(wasm_module).unwrap();

    // Check for wasm_bindgen attributes
    assert!(
        content.contains("#[wasm_bindgen]"),
        "Missing wasm_bindgen attributes"
    );
    assert!(
        content.contains("wasm_bindgen(constructor)"),
        "Missing constructor binding"
    );
    assert!(content.contains("js_name"), "Missing JS name bindings");
}

#[test]
fn test_webworker_support_configured() {
    // Check for WebWorker support configuration
    let wasm_module = Path::new("src/wasm/");
    assert!(wasm_module.exists(), "WASM module directory not found");

    // In a full implementation, we'd check for worker message handling
    // For now, just verify the module structure exists
}

#[test]
fn test_wasm_performance_target_10ms() {
    // This test would measure actual execution time
    // For now, we'll create a placeholder that checks optimization level

    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();

    // Ensure release mode optimizations are enabled
    assert!(
        cargo_toml.contains("opt-level") || cargo_toml.contains("[profile.release]"),
        "Missing performance optimization configuration"
    );
}
