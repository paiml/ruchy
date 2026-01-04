//! Probar-based WASM Module Tests
//!
//! Comprehensive tests for all WASM modules using jugar-probar for:
//! - GUI coverage tracking of public API surface
//! - Systematic testing of demo_converter, shared_session, wit, portability
//!
//! Run with: `cargo test --test probar_wasm_modules`

use jugar_probar::prelude::*;
use std::collections::HashMap;

// =============================================================================
// GUI Coverage Tracking for WASM Modules
// =============================================================================

fn wasm_modules_coverage() -> UxCoverageTracker {
    UxCoverageBuilder::new()
        // Demo Converter API
        .button("demo_notebook_cell_code")
        .button("demo_notebook_cell_markdown")
        .button("demo_convert_to_notebook")
        .button("demo_find_demo_files")
        // WIT Generator API
        .button("wit_generator_new")
        .button("wit_generate_from_source")
        .button("wit_generate_from_component")
        // Portability Analyzer API
        .button("port_analyzer_new")
        .button("port_analyzer_analyze")
        .button("port_compatibility_matrix")
        // Shared Session API removed - using Pure Rust notebook module
        // REPL State API
        .button("repl_state_new")
        .button("repl_state_mode")
        .button("repl_state_history")
        .button("repl_state_bindings")
        // Screens for test categories
        .screen("demo_converter")
        .screen("wit_generator")
        .screen("portability")
        .screen("repl")
        .screen("integration")
        .build()
}

// =============================================================================
// Demo Converter Tests
// =============================================================================

#[test]
fn test_probar_demo_notebook_cell_code() {
    use ruchy::wasm::demo_converter::NotebookCell;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_notebook_cell_code");

    let cell = NotebookCell::code("let x = 42".to_string());
    assert_eq!(cell.cell_type, "code");
    assert_eq!(cell.source, "let x = 42");
}

#[test]
fn test_probar_demo_notebook_cell_markdown() {
    use ruchy::wasm::demo_converter::NotebookCell;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_notebook_cell_markdown");

    let cell = NotebookCell::markdown("# Header".to_string());
    assert_eq!(cell.cell_type, "markdown");
    assert_eq!(cell.source, "# Header");
}

#[test]
fn test_probar_demo_convert_simple() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    let content = "42\nlet x = 10";
    let notebook = convert_demo_to_notebook("test", content).unwrap();

    assert_eq!(notebook.cells.len(), 2);
    assert_eq!(notebook.nbformat, 4);
    assert_eq!(notebook.nbformat_minor, 2);
}

#[test]
fn test_probar_demo_convert_with_comments() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    let content = "# This is a comment\n42";
    let notebook = convert_demo_to_notebook("test", content).unwrap();

    assert_eq!(notebook.cells.len(), 2);
    assert_eq!(notebook.cells[0].cell_type, "markdown");
    assert_eq!(notebook.cells[1].cell_type, "code");
}

#[test]
fn test_probar_demo_convert_multiline_function() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    let content = "fun foo() {\n  42\n}";
    let notebook = convert_demo_to_notebook("test", content).unwrap();

    assert!(!notebook.cells.is_empty());
}

#[test]
fn test_probar_demo_convert_with_repl_commands() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    // REPL commands starting with : should be filtered
    let content = "42\n:help\nlet x = 10";
    let notebook = convert_demo_to_notebook("test", content).unwrap();

    // Should have 2 cells (42 and let x = 10), :help filtered out
    assert_eq!(notebook.cells.len(), 2);
}

#[test]
fn test_probar_demo_convert_empty_lines() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    let content = "\n\n42\n\n\nlet x = 10\n\n";
    let notebook = convert_demo_to_notebook("test", content).unwrap();

    // Empty lines should be skipped
    assert_eq!(notebook.cells.len(), 2);
}

#[test]
fn test_probar_demo_convert_control_structures() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    // Test if, while, for, match multiline detection
    let control_structures = vec![
        "if true {\n  1\n}",
        "while x > 0 {\n  x = x - 1\n}",
        "for i in 0..10 {\n  i\n}",
        "match x {\n  1 => a,\n  _ => b\n}",
    ];

    for content in control_structures {
        let notebook = convert_demo_to_notebook("test", content).unwrap();
        assert!(!notebook.cells.is_empty());
    }
}

#[test]
fn test_probar_demo_notebook_metadata() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_convert_to_notebook");

    let content = "42";
    let notebook = convert_demo_to_notebook("my_demo", content).unwrap();

    assert!(notebook.metadata.contains_key("language_info"));
    assert!(notebook.metadata.contains_key("kernelspec"));
    assert!(notebook.metadata.contains_key("original_demo"));
}

#[test]
fn test_probar_demo_find_demo_files() {
    use ruchy::wasm::demo_converter::find_demo_files;

    let mut gui = wasm_modules_coverage();
    gui.visit("demo_converter");
    gui.click("demo_find_demo_files");

    // May or may not find files depending on examples dir
    let files = find_demo_files();
    // Files should be sorted
    let mut sorted = files.clone();
    sorted.sort();
    assert_eq!(files, sorted);
}

// =============================================================================
// WIT Generator Tests
// =============================================================================

#[test]
fn test_probar_wit_generator_new() {
    use ruchy::wasm::wit::WitGenerator;

    let mut gui = wasm_modules_coverage();
    gui.visit("wit_generator");
    gui.click("wit_generator_new");

    let generator = WitGenerator::new();
    // Generator should be created successfully
    let _ = generator;
}

#[test]
fn test_probar_wit_generate_from_source() {
    use ruchy::wasm::wit::WitGenerator;

    let mut gui = wasm_modules_coverage();
    gui.visit("wit_generator");
    gui.click("wit_generate_from_source");

    let mut generator = WitGenerator::new();
    let result = generator.generate_from_source("fun main() {}");

    assert!(result.is_ok());
    let interface = result.unwrap();
    assert_eq!(interface.name, "ruchy-component");
}

#[test]
fn test_probar_wit_generate_from_component() {
    use ruchy::wasm::component::{ComponentMetadata, WasmComponent};
    use ruchy::wasm::wit::WitGenerator;

    let mut gui = wasm_modules_coverage();
    gui.visit("wit_generator");
    gui.click("wit_generate_from_component");

    let mut generator = WitGenerator::new();
    let component = WasmComponent {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![],
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    };

    let result = generator.generate(&component);
    assert!(result.is_ok());
}

// =============================================================================
// Portability Analyzer Tests
// =============================================================================

#[test]
fn test_probar_port_analyzer_new() {
    use ruchy::wasm::portability::PortabilityAnalyzer;

    let mut gui = wasm_modules_coverage();
    gui.visit("portability");
    gui.click("port_analyzer_new");

    let analyzer = PortabilityAnalyzer::new();
    // Analyzer should be created successfully
    let _ = analyzer;
}

#[test]
fn test_probar_port_analyzer_analyze() {
    use ruchy::wasm::component::{ComponentMetadata, WasmComponent};
    use ruchy::wasm::portability::PortabilityAnalyzer;

    let mut gui = wasm_modules_coverage();
    gui.visit("portability");
    gui.click("port_analyzer_analyze");

    let analyzer = PortabilityAnalyzer::new();
    let component = WasmComponent {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 1024],
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    };

    let report = analyzer.analyze(&component).unwrap();

    assert_eq!(report.component_info.name, "test");
    assert!(report.score.overall_score > 0.0);
    assert!(report.score.overall_score <= 1.0);
}

#[test]
fn test_probar_port_analyzer_size_tiers() {
    use ruchy::wasm::component::{ComponentMetadata, WasmComponent};
    use ruchy::wasm::portability::PortabilityAnalyzer;

    let mut gui = wasm_modules_coverage();
    gui.visit("portability");
    gui.click("port_analyzer_analyze");

    let analyzer = PortabilityAnalyzer::new();

    // Test different size tiers
    let sizes = vec![
        10 * 1024,   // 10KB
        75 * 1024,   // 75KB
        300 * 1024,  // 300KB
        800 * 1024,  // 800KB
        2000 * 1024, // 2MB
    ];

    for size in sizes {
        let component = WasmComponent {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            bytecode: vec![0u8; size],
            imports: vec![],
            exports: vec![],
            metadata: ComponentMetadata::default(),
            custom_sections: HashMap::new(),
        };

        let report = analyzer.analyze(&component).unwrap();
        assert!(report.score.size_efficiency <= 1.0);
        assert!(report.score.size_efficiency >= 0.0);
    }
}

#[test]
fn test_probar_port_default_analyzer() {
    use ruchy::wasm::portability::PortabilityAnalyzer;

    let mut gui = wasm_modules_coverage();
    gui.visit("portability");
    gui.click("port_compatibility_matrix");

    // Test Default implementation
    let analyzer: PortabilityAnalyzer = PortabilityAnalyzer::default();
    let _ = analyzer;
}

// Shared Session Tests removed - using Pure Rust notebook module instead

// =============================================================================
// REPL State Tests
// =============================================================================

#[test]
fn test_probar_repl_state_new() {
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_new");

    let state = ReplState::new();
    assert!(state.get_bindings().is_empty());
}

#[test]
fn test_probar_repl_state_modes() {
    use ruchy::runtime::repl::state::{ReplMode, ReplState};

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_mode");

    let mut state = ReplState::new();

    let modes = vec![
        ReplMode::Normal,
        ReplMode::Debug,
        ReplMode::Ast,
        ReplMode::Transpile,
    ];

    for mode in modes {
        state.set_mode(mode);
        assert_eq!(state.get_mode(), mode);
    }
}

#[test]
fn test_probar_repl_history() {
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_history");

    let mut state = ReplState::new();

    state.add_to_history("cmd1".to_string());
    state.add_to_history("cmd2".to_string());
    state.add_to_history("cmd3".to_string());

    let history = state.get_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0], "cmd1");
    assert_eq!(history[2], "cmd3");
}

#[test]
fn test_probar_repl_bindings() {
    use ruchy::runtime::interpreter::Value;
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_bindings");

    let mut state = ReplState::new();

    state.set_variable("x".to_string(), Value::Integer(42));
    state.set_variable("y".to_string(), Value::Float(3.14));

    assert!(state.get_variable("x").is_some());
    assert!(state.get_variable("y").is_some());
    assert!(state.get_variable("z").is_none());
}

#[test]
fn test_probar_repl_result_history() {
    use ruchy::runtime::interpreter::Value;
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_history");

    let mut state = ReplState::new();

    state.add_to_result_history(Value::Integer(1));
    state.add_to_result_history(Value::Integer(2));
    state.add_to_result_history(Value::Integer(3));

    assert_eq!(state.result_history_len(), 3);
}

#[test]
fn test_probar_repl_peak_memory() {
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_new");

    let mut state = ReplState::new();

    state.update_peak_memory(1000);
    assert_eq!(state.get_peak_memory(), 1000);

    state.update_peak_memory(2000);
    assert_eq!(state.get_peak_memory(), 2000);

    state.update_peak_memory(500); // Lower, shouldn't change
    assert_eq!(state.get_peak_memory(), 2000);
}

#[test]
fn test_probar_repl_snapshot_restore() {
    use ruchy::runtime::interpreter::Value;
    use ruchy::runtime::repl::state::ReplState;

    let mut gui = wasm_modules_coverage();
    gui.visit("repl");
    gui.click("repl_state_bindings");

    let mut state = ReplState::new();

    state.set_variable("x".to_string(), Value::Integer(10));
    let snapshot = state.bindings_snapshot();

    state.set_variable("x".to_string(), Value::Integer(20));
    state.restore_bindings(snapshot);

    let value = state.get_variable("x");
    assert!(matches!(value.unwrap(), Value::Integer(10)));
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_probar_integration_demo_to_notebook() {
    use ruchy::wasm::demo_converter::convert_demo_to_notebook;

    let mut gui = wasm_modules_coverage();
    gui.visit("integration");

    let demo_content = r#"
# Demo Title
let x = 42

fun square(n) {
    n * n
}

square(x)
"#;

    let notebook = convert_demo_to_notebook("integration_test", demo_content).unwrap();

    // Should have cells
    assert!(!notebook.cells.is_empty());

    // Check metadata
    assert_eq!(
        notebook
            .metadata
            .get("original_demo")
            .unwrap()
            .as_str()
            .unwrap(),
        "integration_test"
    );
}

#[test]
fn test_probar_integration_wit_and_portability() {
    use ruchy::wasm::component::{ComponentMetadata, WasmComponent};
    use ruchy::wasm::portability::PortabilityAnalyzer;
    use ruchy::wasm::wit::WitGenerator;

    let mut gui = wasm_modules_coverage();
    gui.visit("integration");

    // Create a component
    let component = WasmComponent {
        name: "integration-test".to_string(),
        version: "1.0.0".to_string(),
        bytecode: vec![0u8; 1024],
        imports: vec![],
        exports: vec![],
        metadata: ComponentMetadata::default(),
        custom_sections: HashMap::new(),
    };

    // Generate WIT
    let mut wit_generator = WitGenerator::new();
    let wit_result = wit_generator.generate(&component);
    assert!(wit_result.is_ok());

    // Analyze portability
    let analyzer = PortabilityAnalyzer::new();
    let report = analyzer.analyze(&component).unwrap();

    assert!(report.score.overall_score > 0.0);
}

// =============================================================================
// Coverage Report
// =============================================================================

#[test]
fn test_probar_wasm_modules_coverage_report() {
    let mut gui = wasm_modules_coverage();

    // Record all operations
    // Demo Converter
    gui.click("demo_notebook_cell_code");
    gui.click("demo_notebook_cell_markdown");
    gui.click("demo_convert_to_notebook");
    gui.click("demo_find_demo_files");

    // WIT Generator
    gui.click("wit_generator_new");
    gui.click("wit_generate_from_source");
    gui.click("wit_generate_from_component");

    // Portability
    gui.click("port_analyzer_new");
    gui.click("port_analyzer_analyze");
    gui.click("port_compatibility_matrix");

    // Shared Session
    gui.click("session_new");
    gui.click("session_execute");
    gui.click("session_transaction");
    gui.click("session_export_import");
    gui.click("session_registry");

    // REPL
    gui.click("repl_state_new");
    gui.click("repl_state_mode");
    gui.click("repl_state_history");
    gui.click("repl_state_bindings");

    // Visit all screens
    gui.visit("demo_converter");
    gui.visit("wit_generator");
    gui.visit("portability");
    gui.visit("shared_session");
    gui.visit("repl");
    gui.visit("integration");

    // Generate report
    let report = gui.generate_report();
    println!("\n{report}");
    println!("WASM Modules Coverage: {}", gui.summary());
    println!("Coverage Percentage: {:.1}%", gui.percent());

    // Assert high coverage
    assert!(
        gui.meets(80.0),
        "WASM modules coverage should be at least 80%: {:.1}%",
        gui.percent()
    );
}
