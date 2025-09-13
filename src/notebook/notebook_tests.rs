//! Comprehensive TDD tests for Notebook module
//! Target: Increase coverage from 2% to 40%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod notebook_tests {
    use crate::notebook::{Notebook, NotebookCell, CellType, CellOutput, NotebookMetadata};
    use std::collections::HashMap;
    
    // ========== Notebook Creation Tests ==========
    
    #[test]
    fn test_notebook_creation() {
        let notebook = Notebook::new();
        assert_eq!(notebook.version(), "1.0.0");
        assert_eq!(notebook.cell_count(), 0);
        assert!(notebook.is_empty());
    }
    
    #[test]
    fn test_notebook_with_metadata() {
        let mut metadata = NotebookMetadata::default();
        metadata.title = "Test Notebook".to_string();
        metadata.author = "Test Author".to_string();
        
        let notebook = Notebook::with_metadata(metadata.clone());
        assert_eq!(notebook.metadata().title, "Test Notebook");
        assert_eq!(notebook.metadata().author, "Test Author");
    }
    
    // ========== Cell Management Tests ==========
    
    #[test]
    fn test_add_code_cell() {
        let mut notebook = Notebook::new();
        let cell = NotebookCell::new_code("print('Hello, World!')");
        let id = notebook.add_cell(cell);
        
        assert_eq!(notebook.cell_count(), 1);
        assert!(!notebook.is_empty());
        assert!(id.len() > 0);
    }
    
    #[test]
    fn test_add_markdown_cell() {
        let mut notebook = Notebook::new();
        let cell = NotebookCell::new_markdown("# Test Header\n\nThis is a test.");
        let id = notebook.add_cell(cell);
        
        let retrieved = notebook.get_cell(&id).unwrap();
        assert_eq!(retrieved.cell_type, CellType::Markdown);
        assert_eq!(retrieved.source, "# Test Header\n\nThis is a test.");
    }
    
    #[test]
    fn test_add_raw_cell() {
        let mut notebook = Notebook::new();
        let cell = NotebookCell::new_raw("Raw text content");
        let id = notebook.add_cell(cell);
        
        let retrieved = notebook.get_cell(&id).unwrap();
        assert_eq!(retrieved.cell_type, CellType::Raw);
    }
    
    #[test]
    fn test_insert_cell_at_index() {
        let mut notebook = Notebook::new();
        notebook.add_cell(NotebookCell::new_code("cell 1"));
        notebook.add_cell(NotebookCell::new_code("cell 2"));
        
        let cell = NotebookCell::new_code("inserted");
        notebook.insert_cell_at(1, cell);
        
        assert_eq!(notebook.cell_count(), 3);
        let cells = notebook.get_all_cells();
        assert_eq!(cells[1].source, "inserted");
    }
    
    #[test]
    fn test_remove_cell() {
        let mut notebook = Notebook::new();
        let id1 = notebook.add_cell(NotebookCell::new_code("cell 1"));
        let id2 = notebook.add_cell(NotebookCell::new_code("cell 2"));
        
        assert_eq!(notebook.cell_count(), 2);
        
        let removed = notebook.remove_cell(&id1);
        assert!(removed);
        assert_eq!(notebook.cell_count(), 1);
        assert!(notebook.get_cell(&id1).is_none());
        assert!(notebook.get_cell(&id2).is_some());
    }
    
    #[test]
    fn test_move_cell() {
        let mut notebook = Notebook::new();
        let id1 = notebook.add_cell(NotebookCell::new_code("cell 1"));
        let id2 = notebook.add_cell(NotebookCell::new_code("cell 2"));
        let id3 = notebook.add_cell(NotebookCell::new_code("cell 3"));
        
        notebook.move_cell(&id3, 0);
        
        let cells = notebook.get_all_cells();
        assert_eq!(cells[0].source, "cell 3");
        assert_eq!(cells[1].source, "cell 1");
        assert_eq!(cells[2].source, "cell 2");
    }
    
    // ========== Cell Execution Tests ==========
    
    #[test]
    fn test_execute_code_cell() {
        let mut notebook = Notebook::new();
        let id = notebook.add_cell(NotebookCell::new_code("1 + 1"));
        
        let output = notebook.execute_cell(&id);
        assert!(output.is_some());
        
        let output = output.unwrap();
        assert!(output.success);
        assert!(output.execution_time >= 0.0);
    }
    
    #[test]
    fn test_execute_with_error() {
        let mut notebook = Notebook::new();
        let id = notebook.add_cell(NotebookCell::new_code("undefined_variable"));
        
        let output = notebook.execute_cell(&id);
        assert!(output.is_some());
        
        let output = output.unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
    }
    
    #[test]
    fn test_execute_all_cells() {
        let mut notebook = Notebook::new();
        notebook.add_cell(NotebookCell::new_code("let x = 10"));
        notebook.add_cell(NotebookCell::new_code("let y = 20"));
        notebook.add_cell(NotebookCell::new_code("x + y"));
        
        let outputs = notebook.execute_all();
        assert_eq!(outputs.len(), 3);
        
        for output in outputs {
            assert!(output.execution_time >= 0.0);
        }
    }
    
    #[test]
    fn test_clear_outputs() {
        let mut notebook = Notebook::new();
        let id = notebook.add_cell(NotebookCell::new_code("print('test')"));
        
        notebook.execute_cell(&id);
        let cell = notebook.get_cell(&id).unwrap();
        assert!(cell.outputs.is_some());
        
        notebook.clear_outputs(&id);
        let cell = notebook.get_cell(&id).unwrap();
        assert!(cell.outputs.is_none());
    }
    
    #[test]
    fn test_clear_all_outputs() {
        let mut notebook = Notebook::new();
        let id1 = notebook.add_cell(NotebookCell::new_code("1 + 1"));
        let id2 = notebook.add_cell(NotebookCell::new_code("2 + 2"));
        
        notebook.execute_cell(&id1);
        notebook.execute_cell(&id2);
        
        notebook.clear_all_outputs();
        
        assert!(notebook.get_cell(&id1).unwrap().outputs.is_none());
        assert!(notebook.get_cell(&id2).unwrap().outputs.is_none());
    }
    
    // ========== Serialization Tests ==========
    
    #[test]
    fn test_to_json() {
        let mut notebook = Notebook::new();
        notebook.add_cell(NotebookCell::new_code("print('test')"));
        
        let json = notebook.to_json();
        assert!(json.is_ok());
        
        let json_str = json.unwrap();
        assert!(json_str.contains("cells"));
        assert!(json_str.contains("metadata"));
        assert!(json_str.contains("version"));
    }
    
    #[test]
    fn test_from_json() {
        let json_str = r#"{
            "version": "1.0.0",
            "metadata": {"title": "Test"},
            "cells": [
                {"cell_type": "code", "source": "1 + 1"}
            ]
        }"#;
        
        let notebook = Notebook::from_json(json_str);
        assert!(notebook.is_ok());
        
        let notebook = notebook.unwrap();
        assert_eq!(notebook.cell_count(), 1);
        assert_eq!(notebook.metadata().title, "Test");
    }
    
    #[test]
    fn test_save_to_file() {
        let mut notebook = Notebook::new();
        notebook.add_cell(NotebookCell::new_code("test code"));
        
        let path = "/tmp/test_notebook.rynb";
        let result = notebook.save(path);
        assert!(result.is_ok());
        
        // Clean up
        std::fs::remove_file(path).ok();
    }
    
    #[test]
    fn test_load_from_file() {
        // First save a notebook
        let mut notebook = Notebook::new();
        notebook.add_cell(NotebookCell::new_code("test code"));
        
        let path = "/tmp/test_notebook.rynb";
        notebook.save(path).unwrap();
        
        // Now load it
        let loaded = Notebook::load(path);
        assert!(loaded.is_ok());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.cell_count(), 1);
        
        // Clean up
        std::fs::remove_file(path).ok();
    }
    
    // ========== Metadata Tests ==========
    
    #[test]
    fn test_metadata_default() {
        let metadata = NotebookMetadata::default();
        assert_eq!(metadata.kernel, "ruchy");
        assert_eq!(metadata.language, "ruchy");
        assert!(metadata.created_at > 0);
    }
    
    #[test]
    fn test_update_metadata() {
        let mut notebook = Notebook::new();
        
        notebook.set_title("My Notebook");
        notebook.set_author("Jane Doe");
        notebook.add_tag("tutorial");
        notebook.add_tag("beginner");
        
        assert_eq!(notebook.metadata().title, "My Notebook");
        assert_eq!(notebook.metadata().author, "Jane Doe");
        assert_eq!(notebook.metadata().tags.len(), 2);
        assert!(notebook.metadata().tags.contains(&"tutorial".to_string()));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl Notebook {
        /// Helper: Check if notebook is empty
        fn is_empty(&self) -> bool {
            self.cell_count() == 0
        }
        
        /// Helper: Get notebook version
        fn version(&self) -> &str {
            &self.version
        }
        
        /// Helper: Get metadata
        fn metadata(&self) -> &NotebookMetadata {
            &self.metadata
        }
    }
    
    impl NotebookCell {
        /// Helper: Create new code cell
        fn new_code(source: &str) -> Self {
            NotebookCell {
                cell_type: CellType::Code,
                source: source.to_string(),
                outputs: None,
                metadata: HashMap::new(),
            }
        }
        
        /// Helper: Create new markdown cell
        fn new_markdown(source: &str) -> Self {
            NotebookCell {
                cell_type: CellType::Markdown,
                source: source.to_string(),
                outputs: None,
                metadata: HashMap::new(),
            }
        }
        
        /// Helper: Create new raw cell
        fn new_raw(source: &str) -> Self {
            NotebookCell {
                cell_type: CellType::Raw,
                source: source.to_string(),
                outputs: None,
                metadata: HashMap::new(),
            }
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_add_cells_never_panics(sources in prop::collection::vec("[a-z]+", 1..50)) {
            let mut notebook = Notebook::new();
            
            for source in sources {
                let cell = NotebookCell::new_code(&source);
                let id = notebook.add_cell(cell);
                assert!(id.len() > 0);
            }
        }
        
        #[test]
        fn test_cell_order_preserved(count in 1usize..20) {
            let mut notebook = Notebook::new();
            let mut ids = Vec::new();
            
            for i in 0..count {
                let cell = NotebookCell::new_code(&format!("cell {}", i));
                let id = notebook.add_cell(cell);
                ids.push(id);
            }
            
            let cells = notebook.get_all_cells();
            for (i, cell) in cells.iter().enumerate() {
                assert_eq!(cell.source, format!("cell {}", i));
            }
        }
        
        #[test]
        fn test_json_roundtrip(cells in prop::collection::vec("[a-z]+", 0..10)) {
            let mut notebook = Notebook::new();
            
            for source in &cells {
                notebook.add_cell(NotebookCell::new_code(source));
            }
            
            let json = notebook.to_json().unwrap();
            let loaded = Notebook::from_json(&json).unwrap();
            
            assert_eq!(notebook.cell_count(), loaded.cell_count());
        }
    }
}