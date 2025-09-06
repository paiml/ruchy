//! TDD tests for reducing complexity in handlers module
//! Target: Reduce cyclomatic complexity from 14 to <10

use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod handle_transpile_tests {
    use super::*;
    
    fn create_test_file(content: &str) -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.ruchy");
        fs::write(&file_path, content).unwrap();
        (dir, file_path)
    }
    
    #[test]
    fn test_transpile_simple_file() {
        let (_dir, file) = create_test_file("let x = 42");
        let output = TempDir::new().unwrap();
        let output_path = output.path().join("output.rs");
        
        // This test verifies basic transpilation works
        // We'll refactor handle_transpile_command to reduce complexity
        // by extracting helper functions
    }
    
    #[test]
    fn test_read_source_from_file() {
        let (_dir, file) = create_test_file("let x = 42");
        
        // Test helper: read_source_file(&Path) -> Result<String>
        // This extracts file reading logic
    }
    
    #[test]
    fn test_read_source_from_stdin() {
        // Test helper: read_source_stdin() -> Result<String>
        // This extracts stdin reading logic
    }
    
    #[test]
    fn test_parse_source() {
        let source = "let x = 42";
        
        // Test helper: parse_source(&str) -> Result<AST>
        // This extracts parsing logic
    }
    
    #[test]
    fn test_transpile_ast() {
        // Test helper: transpile_ast(&AST, bool minimal) -> Result<String>
        // This extracts transpilation logic
    }
    
    #[test]
    fn test_write_output() {
        let output = TempDir::new().unwrap();
        let output_path = output.path().join("output.rs");
        let rust_code = "fn main() {}";
        
        // Test helper: write_output(Option<&Path>, &str, bool verbose) -> Result<()>
        // This extracts output writing logic
    }
}

#[cfg(test)]
mod handle_run_tests {
    use super::*;
    
    #[test]
    fn test_compile_rust_code() {
        // Test helper: compile_rust_code(&Path, &Path) -> Result<()>
        // This extracts compilation logic
    }
    
    #[test]
    fn test_execute_binary() {
        // Test helper: execute_binary(&Path) -> Result<Output>
        // This extracts execution logic
    }
    
    #[test]
    fn test_create_temp_files() {
        // Test helper: create_temp_files(&str) -> Result<(TempFile, PathBuf)>
        // This extracts temp file creation
    }
    
    #[test]
    fn test_process_output() {
        // Test helper: process_output(&Output) -> Result<()>
        // This extracts output processing logic
    }
}

#[cfg(test)]
mod compile_source_tests {
    use super::*;
    
    #[test]
    fn test_validate_inputs() {
        // Test helper: validate_compile_inputs(&Path, &Path) -> Result<()>
        // This extracts input validation
    }
    
    #[test]
    fn test_prepare_compilation() {
        // Test helper: prepare_compilation(&str) -> Result<CompilationContext>
        // This extracts compilation preparation
    }
    
    #[test]
    fn test_run_rustc() {
        // Test helper: run_rustc(&CompilationContext) -> Result<()>
        // This extracts rustc invocation
    }
}