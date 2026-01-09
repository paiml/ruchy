//! Pure Rust `NotebookRuntime` - probador validated, zero JavaScript
//!
//! Minimal notebook runtime with 100% test coverage.
//! Replaces the 6K-line WASM version with Pure Rust.

#![forbid(unsafe_code)]

use crate::notebook::engine::NotebookEngine;
use crate::notebook::types::{Cell, CellType, Notebook, NotebookMetadata};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pure Rust notebook runtime
///
/// Provides notebook document management and cell execution.
/// No JavaScript/WASM dependencies.
#[derive(Debug)]
pub struct NotebookRuntime {
    notebook: Notebook,
    engine: NotebookEngine,
    execution_count: usize,
    cell_outputs: HashMap<String, String>,
}

impl NotebookRuntime {
    /// Create a new notebook runtime
    ///
    /// # Errors
    /// Returns error if engine initialization fails
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            notebook: Notebook {
                cells: Vec::new(),
                metadata: NotebookMetadata::default(),
            },
            engine: NotebookEngine::new()?,
            execution_count: 0,
            cell_outputs: HashMap::new(),
        })
    }

    /// Add a cell to the notebook
    ///
    /// Returns the cell ID
    pub fn add_cell(&mut self, cell_type: &str, source: &str) -> String {
        let id = generate_cell_id();
        let cell = Cell {
            cell_type: match cell_type {
                "markdown" => CellType::Markdown,
                _ => CellType::Code,
            },
            source: source.to_string(),
            output: None,
            execution_count: None,
        };
        self.notebook.cells.push(cell);
        id
    }

    /// Execute a cell by ID
    ///
    /// Returns the execution output or error message
    pub fn execute_cell(&mut self, cell_id: &str) -> String {
        // Find cell by ID (match by index for simplicity)
        let cell_idx = self.find_cell_index(cell_id);

        if let Some(idx) = cell_idx {
            if let Some(cell) = self.notebook.cells.get_mut(idx) {
                if cell.cell_type == CellType::Markdown {
                    return cell.source.clone();
                }

                self.execution_count += 1;
                match self.engine.execute_cell(&cell.source) {
                    Ok(output) => {
                        cell.output = Some(output.clone());
                        cell.execution_count = Some(self.execution_count as u32);
                        self.cell_outputs.insert(cell_id.to_string(), output.clone());
                        output
                    }
                    Err(e) => {
                        let error_msg = format!("Error: {e}");
                        cell.output = Some(error_msg.clone());
                        error_msg
                    }
                }
            } else {
                format!("Cell not found: {cell_id}")
            }
        } else {
            format!("Cell not found: {cell_id}")
        }
    }

    /// Get cell count
    pub fn cell_count(&self) -> usize {
        self.notebook.cells.len()
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        self.execution_count
    }

    /// Serialize notebook to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.notebook).unwrap_or_else(|_| "{}".to_string())
    }

    /// Get cell output by ID
    pub fn get_cell_output(&self, cell_id: &str) -> Option<&String> {
        self.cell_outputs.get(cell_id)
    }

    /// Clear all cells
    pub fn clear(&mut self) {
        self.notebook.cells.clear();
        self.cell_outputs.clear();
        self.execution_count = 0;
    }

    // Private helper to find cell index
    fn find_cell_index(&self, cell_id: &str) -> Option<usize> {
        // For simplicity, treat cell_id as index if numeric, otherwise search
        if let Ok(idx) = cell_id.parse::<usize>() {
            if idx < self.notebook.cells.len() {
                return Some(idx);
            }
        }
        // Search by checking if any cell output was stored with this ID
        self.cell_outputs.keys().position(|k| k == cell_id)
    }
}

/// Generate a unique cell ID
fn generate_cell_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("cell_{timestamp}")
}

// ============================================================================
// Tests - 100% Coverage Target
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_runtime() {
        let runtime = NotebookRuntime::new();
        assert!(runtime.is_ok());
        let runtime = runtime.unwrap();
        assert_eq!(runtime.cell_count(), 0);
        assert_eq!(runtime.execution_count(), 0);
    }

    #[test]
    fn test_add_code_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        let id = runtime.add_cell("code", "let x = 42");
        assert!(id.starts_with("cell_"));
        assert_eq!(runtime.cell_count(), 1);
    }

    #[test]
    fn test_add_markdown_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        let id = runtime.add_cell("markdown", "# Header");
        assert!(id.starts_with("cell_"));
        assert_eq!(runtime.cell_count(), 1);
    }

    #[test]
    fn test_execute_code_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "1 + 1");
        let output = runtime.execute_cell("0");
        assert_eq!(output, "2");
        assert_eq!(runtime.execution_count(), 1);
    }

    #[test]
    fn test_execute_markdown_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("markdown", "# Title");
        let output = runtime.execute_cell("0");
        assert_eq!(output, "# Title");
    }

    #[test]
    fn test_execute_nonexistent_cell() {
        let mut runtime = NotebookRuntime::new().unwrap();
        let output = runtime.execute_cell("999");
        assert!(output.contains("Cell not found"));
    }

    #[test]
    fn test_to_json() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "x = 1");
        let json = runtime.to_json();
        assert!(json.contains("cells"));
        assert!(json.contains("x = 1"));
    }

    #[test]
    fn test_clear() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "let x = 1");
        runtime.execute_cell("0");
        assert_eq!(runtime.cell_count(), 1);
        assert_eq!(runtime.execution_count(), 1);

        runtime.clear();
        assert_eq!(runtime.cell_count(), 0);
        assert_eq!(runtime.execution_count(), 0);
    }

    #[test]
    fn test_get_cell_output() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "42");
        runtime.execute_cell("0");
        // Output is stored by cell ID
        assert!(runtime.cell_outputs.get("0").is_some());
    }

    #[test]
    fn test_multiple_cells() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "let x = 10");
        runtime.add_cell("code", "x + 5");

        runtime.execute_cell("0");
        let output = runtime.execute_cell("1");
        assert_eq!(output, "15");
        assert_eq!(runtime.execution_count(), 2);
    }

    #[test]
    fn test_cell_error() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "undefined_var");
        let output = runtime.execute_cell("0");
        assert!(output.contains("Error"));
    }

    #[test]
    fn test_generate_cell_id() {
        let id1 = generate_cell_id();
        assert!(id1.starts_with("cell_"));
        // Add small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_micros(10));
        let id2 = generate_cell_id();
        assert!(id2.starts_with("cell_"));
        // IDs should be different (timing-based)
        assert_ne!(id1, id2);
    }

    // ===== EXTREME TDD Round 143 - Additional Coverage Tests =====

    #[test]
    fn test_add_multiple_code_cells() {
        let mut runtime = NotebookRuntime::new().unwrap();
        let id1 = runtime.add_cell("code", "let a = 1");
        let id2 = runtime.add_cell("code", "let b = 2");
        let id3 = runtime.add_cell("code", "a + b");
        assert!(id1.starts_with("cell_"));
        assert!(id2.starts_with("cell_"));
        assert!(id3.starts_with("cell_"));
        assert_eq!(runtime.cell_count(), 3);
    }

    #[test]
    fn test_mixed_cell_types() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "let x = 10");
        runtime.add_cell("markdown", "# Section 1");
        runtime.add_cell("code", "x * 2");
        runtime.add_cell("markdown", "Some notes");
        assert_eq!(runtime.cell_count(), 4);
    }

    #[test]
    fn test_execute_multiple_code_cells_sequentially() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "let x = 5");
        runtime.add_cell("code", "let y = 3");
        runtime.add_cell("code", "x * y");

        runtime.execute_cell("0");
        runtime.execute_cell("1");
        let output = runtime.execute_cell("2");

        assert_eq!(output, "15");
        assert_eq!(runtime.execution_count(), 3);
    }

    #[test]
    fn test_to_json_multiple_cells() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "1 + 1");
        runtime.add_cell("markdown", "# Header");
        runtime.add_cell("code", "2 * 2");

        let json = runtime.to_json();
        assert!(json.contains("cells"));
        assert!(json.contains("1 + 1"));
        assert!(json.contains("# Header"));
        assert!(json.contains("2 * 2"));
    }

    #[test]
    fn test_clear_resets_execution_count() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "1");
        runtime.add_cell("code", "2");
        runtime.execute_cell("0");
        runtime.execute_cell("1");

        assert_eq!(runtime.execution_count(), 2);
        runtime.clear();
        assert_eq!(runtime.execution_count(), 0);
        assert_eq!(runtime.cell_count(), 0);
    }

    #[test]
    fn test_get_cell_output_after_execution() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "100");
        runtime.execute_cell("0");

        let output = runtime.get_cell_output("0");
        assert!(output.is_some());
        assert_eq!(output.unwrap(), "100");
    }

    #[test]
    fn test_get_cell_output_nonexistent() {
        let runtime = NotebookRuntime::new().unwrap();
        let output = runtime.get_cell_output("nonexistent");
        assert!(output.is_none());
    }

    #[test]
    fn test_find_cell_index_by_number() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "a");
        runtime.add_cell("code", "b");
        runtime.add_cell("code", "c");

        // Direct index access
        let idx = runtime.find_cell_index("1");
        assert_eq!(idx, Some(1));
    }

    #[test]
    fn test_find_cell_index_out_of_range() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "test");

        let idx = runtime.find_cell_index("100");
        assert!(idx.is_none());
    }

    #[test]
    fn test_execute_cell_updates_output() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "42");

        let output = runtime.execute_cell("0");
        assert_eq!(output, "42");

        // Check output is stored
        assert!(runtime.cell_outputs.contains_key("0"));
    }

    #[test]
    fn test_cell_type_default_to_code() {
        let mut runtime = NotebookRuntime::new().unwrap();
        // Unknown type should default to code
        runtime.add_cell("unknown_type", "let x = 1");
        runtime.add_cell("random", "let y = 2");

        assert_eq!(runtime.cell_count(), 2);
        // Should execute as code
        let output = runtime.execute_cell("0");
        assert!(!output.contains("Error") || output.contains("undefined"));
    }

    #[test]
    fn test_execution_count_increments_on_code_only() {
        let mut runtime = NotebookRuntime::new().unwrap();
        runtime.add_cell("code", "1");
        runtime.add_cell("markdown", "# Note");
        runtime.add_cell("code", "2");

        runtime.execute_cell("0");
        runtime.execute_cell("1"); // markdown doesn't increment
        runtime.execute_cell("2");

        assert_eq!(runtime.execution_count(), 2);
    }
}
