//! NOTEBOOK-009: Notebook data model types
//!
//! Defines the core data structures for Ruchy notebooks:
//! - `Cell` (code or markdown)
//! - `CellType` (enum)
//! - `Notebook` (collection of cells)
//! - `NotebookMetadata`
//!
//! File format: `.rnb` (Ruchy Notebook) - JSON serialization
//! Future: `.ipynb` compatibility for Jupyter ecosystem

use serde::{Deserialize, Serialize};

/// Type of notebook cell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    /// Executable Ruchy code
    Code,
    /// Markdown documentation
    Markdown,
}

/// A single cell in a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Type of cell (code or markdown)
    pub cell_type: CellType,
    /// Source code or markdown content
    pub source: String,
    /// Output from execution (for code cells only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Execution count (for code cells only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u32>,
}

impl Cell {
    /// Create a new code cell
    pub fn code(source: impl Into<String>) -> Self {
        Self {
            cell_type: CellType::Code,
            source: source.into(),
            output: None,
            execution_count: None,
        }
    }

    /// Create a new markdown cell
    pub fn markdown(source: impl Into<String>) -> Self {
        Self {
            cell_type: CellType::Markdown,
            source: source.into(),
            output: None,
            execution_count: None,
        }
    }

    /// Check if this is a code cell
    pub fn is_code(&self) -> bool {
        self.cell_type == CellType::Code
    }

    /// Check if this is a markdown cell
    pub fn is_markdown(&self) -> bool {
        self.cell_type == CellType::Markdown
    }
}

/// Metadata for a notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    /// Language name
    pub language: String,
    /// Notebook format version
    pub version: String,
    /// Kernel name
    pub kernel: String,
}

impl Default for NotebookMetadata {
    fn default() -> Self {
        Self {
            language: "ruchy".to_string(),
            version: "1.0.0".to_string(),
            kernel: "ruchy".to_string(),
        }
    }
}

/// A Ruchy notebook containing multiple cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    /// List of cells in execution order
    pub cells: Vec<Cell>,
    /// Notebook metadata
    pub metadata: NotebookMetadata,
}

impl Notebook {
    /// Create a new empty notebook
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            metadata: NotebookMetadata::default(),
        }
    }

    /// Add a cell to the notebook
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    /// Count code cells
    pub fn code_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_code()).count()
    }

    /// Count markdown cells
    pub fn markdown_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_markdown()).count()
    }

    /// Get all code cells
    pub fn code_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(|c| c.is_code())
    }

    /// Get all markdown cells
    pub fn markdown_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(|c| c.is_markdown())
    }
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_type_serialization() {
        let code = CellType::Code;
        let json = serde_json::to_string(&code).expect("operation should succeed in test");
        assert_eq!(json, r#""code""#);

        let markdown = CellType::Markdown;
        let json = serde_json::to_string(&markdown).expect("operation should succeed in test");
        assert_eq!(json, r#""markdown""#);
    }

    #[test]
    fn test_cell_type_deserialization() {
        let code: CellType =
            serde_json::from_str(r#""code""#).expect("operation should succeed in test");
        assert_eq!(code, CellType::Code);

        let markdown: CellType =
            serde_json::from_str(r#""markdown""#).expect("operation should succeed in test");
        assert_eq!(markdown, CellType::Markdown);
    }

    #[test]
    fn test_cell_code_constructor() {
        let cell = Cell::code("println(42)");
        assert!(cell.is_code());
        assert!(!cell.is_markdown());
        assert_eq!(cell.source, "println(42)");
        assert!(cell.output.is_none());
        assert!(cell.execution_count.is_none());
    }

    #[test]
    fn test_cell_markdown_constructor() {
        let cell = Cell::markdown("# Hello");
        assert!(cell.is_markdown());
        assert!(!cell.is_code());
        assert_eq!(cell.source, "# Hello");
        assert!(cell.output.is_none());
        assert!(cell.execution_count.is_none());
    }

    #[test]
    fn test_cell_serialization() {
        let cell = Cell::code("42");
        let json = serde_json::to_string(&cell).expect("operation should succeed in test");
        assert!(json.contains(r#""cell_type":"code""#));
        assert!(json.contains(r#""source":"42""#));
        // output and execution_count should be omitted when None
        assert!(!json.contains(r#""output""#));
        assert!(!json.contains(r#""execution_count""#));
    }

    #[test]
    fn test_cell_with_output_serialization() {
        let mut cell = Cell::code("42");
        cell.output = Some("42".to_string());
        cell.execution_count = Some(1);

        let json = serde_json::to_string(&cell).expect("operation should succeed in test");
        assert!(json.contains(r#""output":"42""#));
        assert!(json.contains(r#""execution_count":1"#));
    }

    #[test]
    fn test_cell_deserialization() {
        let json = r#"{"cell_type":"code","source":"42"}"#;
        let cell: Cell = serde_json::from_str(json).expect("operation should succeed in test");
        assert_eq!(cell.cell_type, CellType::Code);
        assert_eq!(cell.source, "42");
        assert!(cell.output.is_none());
    }

    #[test]
    fn test_notebook_metadata_default() {
        let metadata = NotebookMetadata::default();
        assert_eq!(metadata.language, "ruchy");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.kernel, "ruchy");
    }

    #[test]
    fn test_notebook_new() {
        let notebook = Notebook::new();
        assert_eq!(notebook.cells.len(), 0);
        assert_eq!(notebook.metadata.language, "ruchy");
    }

    #[test]
    fn test_notebook_add_cell() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Chapter 1"));
        notebook.add_cell(Cell::code("42"));

        assert_eq!(notebook.cells.len(), 2);
        assert!(notebook.cells[0].is_markdown());
        assert!(notebook.cells[1].is_code());
    }

    #[test]
    fn test_notebook_cell_counts() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Title"));
        notebook.add_cell(Cell::code("1 + 1"));
        notebook.add_cell(Cell::markdown("## Section"));
        notebook.add_cell(Cell::code("2 + 2"));
        notebook.add_cell(Cell::code("3 + 3"));

        assert_eq!(notebook.code_cell_count(), 3);
        assert_eq!(notebook.markdown_cell_count(), 2);
    }

    #[test]
    fn test_notebook_serialization() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Hello"));
        notebook.add_cell(Cell::code("println(42)"));

        let json =
            serde_json::to_string_pretty(&notebook).expect("operation should succeed in test");
        assert!(json.contains(r#""cells""#));
        assert!(json.contains(r#""metadata""#));
        assert!(json.contains(r#""language": "ruchy""#));
    }

    #[test]
    fn test_notebook_deserialization() {
        let json = r##"{
            "cells": [
                {"cell_type": "markdown", "source": "# Title"},
                {"cell_type": "code", "source": "42"}
            ],
            "metadata": {
                "language": "ruchy",
                "version": "1.0.0",
                "kernel": "ruchy"
            }
        }"##;

        let notebook: Notebook =
            serde_json::from_str(json).expect("operation should succeed in test");
        assert_eq!(notebook.cells.len(), 2);
        assert!(notebook.cells[0].is_markdown());
        assert!(notebook.cells[1].is_code());
        assert_eq!(notebook.metadata.language, "ruchy");
    }

    #[test]
    fn test_notebook_round_trip() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Chapter 1: Literals"));
        notebook.add_cell(Cell::code("42"));
        notebook.add_cell(Cell::markdown("This returns 42"));

        // Serialize
        let json = serde_json::to_string(&notebook).expect("operation should succeed in test");

        // Deserialize
        let notebook2: Notebook =
            serde_json::from_str(&json).expect("operation should succeed in test");

        // Verify
        assert_eq!(notebook2.cells.len(), 3);
        assert_eq!(notebook2.code_cell_count(), 1);
        assert_eq!(notebook2.markdown_cell_count(), 2);
    }

    #[test]
    fn test_code_cells_iterator() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Title"));
        notebook.add_cell(Cell::code("1 + 1"));
        notebook.add_cell(Cell::markdown("## Section"));
        notebook.add_cell(Cell::code("2 + 2"));

        let code_sources: Vec<&str> = notebook.code_cells().map(|c| c.source.as_str()).collect();

        assert_eq!(code_sources, vec!["1 + 1", "2 + 2"]);
    }

    #[test]
    fn test_markdown_cells_iterator() {
        let mut notebook = Notebook::new();
        notebook.add_cell(Cell::markdown("# Title"));
        notebook.add_cell(Cell::code("42"));
        notebook.add_cell(Cell::markdown("## Section"));

        let md_sources: Vec<&str> = notebook
            .markdown_cells()
            .map(|c| c.source.as_str())
            .collect();

        assert_eq!(md_sources, vec!["# Title", "## Section"]);
    }
}
