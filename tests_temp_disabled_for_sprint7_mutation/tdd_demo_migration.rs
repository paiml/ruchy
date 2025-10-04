//! TDD tests for demo migration to notebook format
//!
//! Tests conversion of 28+ ruchy demo files to notebook format
//! following TDD RED->GREEN->REFACTOR methodology

use ruchy::wasm::demo_converter::{convert_demo_to_notebook, find_demo_files};
use ruchy::wasm::notebook::NotebookRuntime;
use std::fs;

#[test]
fn test_demo_parser_exists() {
    // Check that we have a demo parser module available
    // This will fail until we implement the parser

    // For now, just verify the demo files exist
    let demo_files = find_demo_files();
    assert!(!demo_files.is_empty(), "No demo files found");
    assert!(
        demo_files.len() >= 28,
        "Expected at least 28 demo files, found {}",
        demo_files.len()
    );
}

#[test]
fn test_simple_demo_to_notebook_conversion() {
    // Check converting a simple demo file to notebook format
    let demo_content = r#"# Simple demo
42
let x = 10
x + 2"#;

    // This should convert to a notebook with 3 cells
    let notebook = convert_demo_to_notebook("simple_demo", demo_content).unwrap();

    // Should have 4 cells: 1 comment + 3 code cells
    assert_eq!(notebook.cells.len(), 4, "Expected 4 cells");
    assert_eq!(notebook.cells[0].cell_type, "markdown");
    assert_eq!(notebook.cells[0].source, "# Simple demo");
    assert_eq!(notebook.cells[1].source, "42");
    assert_eq!(notebook.cells[2].source, "let x = 10");
    assert_eq!(notebook.cells[3].source, "x + 2");
}

#[test]
fn test_demo_with_comments_conversion() {
    let demo_content = r#"# This is a comment
# Another comment
42
# More comments
let x = 5"#;

    let notebook = convert_demo_to_notebook("comment_demo", demo_content).unwrap();

    // Comments should become markdown cells, code should become code cells
    assert_eq!(notebook.cells.len(), 5, "Expected 5 cells");
    assert_eq!(notebook.cells[0].cell_type, "markdown");
    assert_eq!(notebook.cells[1].cell_type, "markdown");
    assert_eq!(notebook.cells[2].cell_type, "code");
    assert_eq!(notebook.cells[3].cell_type, "markdown");
    assert_eq!(notebook.cells[4].cell_type, "code");
}

#[test]
fn test_demo_with_repl_commands_conversion() {
    let demo_content = r#"42
:rust 1 + 2
:ast if true { 1 } else { 0 }
let x = 10"#;

    let notebook = convert_demo_to_notebook("repl_demo", demo_content).unwrap();

    // REPL commands should be filtered out or converted to special cells
    assert_eq!(
        notebook.cells.len(),
        2,
        "Expected 2 cells (filtering REPL commands)"
    );
    assert_eq!(notebook.cells[0].source, "42");
    assert_eq!(notebook.cells[1].source, "let x = 10");
}

#[test]
fn test_multiline_statements_conversion() {
    let demo_content = r#"fun add(a, b) {
    a + b
}

if true {
    42
} else {
    0
}"#;

    let notebook = convert_demo_to_notebook("multiline_demo", demo_content).unwrap();

    // Multi-line statements should be single cells
    assert_eq!(notebook.cells.len(), 2, "Expected 2 cells");
    assert!(notebook.cells[0].source.contains("fun add(a, b)"));
    assert!(notebook.cells[1].source.contains("if true"));
}

#[test]
fn test_notebook_execution_after_conversion() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Convert a simple demo and execute it
    let demo_content = "let x = 42\nx + 8";
    let notebook = convert_demo_to_notebook("exec_demo", demo_content).unwrap();

    // Execute first cell
    let result1 = runtime
        .execute_cell_with_session("cell1", &notebook.cells[0].source)
        .unwrap();
    // Assignment expressions return nil, not the assigned value
    assert_eq!(result1.value, "nil");

    // Execute second cell - should have access to x from first cell
    let result2 = runtime
        .execute_cell_with_session("cell2", &notebook.cells[1].source)
        .unwrap();
    assert_eq!(result2.value, "50");
}

#[test]
fn test_all_demo_files_convertible() {
    // Check that all 34 demo files can be converted without errors
    let demo_files = find_demo_files();
    let mut conversion_count = 0;

    for demo_path in demo_files {
        let content = fs::read_to_string(&demo_path).unwrap();
        let filename = demo_path.file_stem().unwrap().to_str().unwrap();

        // This should not panic or return error
        match convert_demo_to_notebook(filename, &content) {
            Ok(_) => conversion_count += 1,
            Err(e) => panic!("Failed to convert {}: {}", demo_path.display(), e),
        }
    }

    assert!(
        conversion_count >= 28,
        "Expected to convert at least 28 demos, got {}",
        conversion_count
    );
}

#[test]
fn test_notebook_format_compliance() {
    // Check that converted notebooks match expected JSON structure
    let demo_content = "42\nlet x = 10";
    let notebook = convert_demo_to_notebook("format_test", demo_content).unwrap();

    // Serialize to JSON and verify structure
    let json = serde_json::to_value(&notebook).unwrap();

    assert!(json["cells"].is_array());
    assert!(json["metadata"].is_object());
    assert_eq!(json["nbformat"], 4);
    assert_eq!(json["nbformat_minor"], 2);
}

#[test]
fn test_notebook_metadata_generation() {
    let notebook = convert_demo_to_notebook("metadata_test", "42").unwrap();

    // Check metadata fields
    assert_eq!(
        notebook
            .metadata
            .get("language_info")
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("name"))
            .and_then(|n| n.as_str()),
        Some("ruchy")
    );

    assert!(notebook.metadata.contains_key("kernelspec"));
}
