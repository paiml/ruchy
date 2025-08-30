// STDLIB-006: File Operations Test Suite
// Following Toyota Way TDD - RED phase first

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;
use std::path::Path;

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {e:?}"))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {e:?}"))?;
    
    Ok(result)
}

// Helper to test transpiled code with unique filenames
fn eval_transpiled(code: &str) -> Result<String, String> {
    let test_file = format!("/tmp/file_ops_test_{}.ruchy", 
        std::process::id());
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {e}"))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {e}"))?;
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    
    if !output.status.success() {
        return Err(format!("Execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[test]
fn test_file_exists() {
    let test_file = "/tmp/test_exists_file.txt";
    
    // Clean up first
    let _ = fs::remove_file(test_file);
    
    // Test file doesn't exist
    let code = format!(r#"file_exists("{test_file}")"#);
    let result = eval_in_repl(&code).unwrap();
    assert_eq!(result, "false");
    
    // Create file
    fs::write(test_file, "test content").unwrap();
    
    // Test file exists 
    let result = eval_in_repl(&code).unwrap();
    assert_eq!(result, "true");
    
    // Test transpiled version
    let code = format!(r#"println(file_exists("{test_file}"))"#);
    let result = eval_transpiled(&code).unwrap();
    assert_eq!(result, "true");
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_append_file() {
    let test_file = "/tmp/test_append_file.txt";
    
    // Clean up first
    let _ = fs::remove_file(test_file);
    
    // Create initial file
    fs::write(test_file, "Initial content\n").unwrap();
    
    // Test append in REPL
    let code = format!(r#"append_file("{test_file}", "Appended line\n")"#);
    let result = eval_in_repl(&code);
    assert!(result.is_ok(), "append_file should work in REPL: {result:?}");
    
    // Verify content
    let content = fs::read_to_string(test_file).unwrap();
    assert!(content.contains("Initial content"));
    assert!(content.contains("Appended line"));
    
    // Test transpiled version
    let code = format!(r#"append_file("{test_file}", "Another line\n")"#);
    let result = eval_transpiled(&code);
    assert!(result.is_ok(), "append_file should work in transpiler: {result:?}");
    
    // Verify final content
    let content = fs::read_to_string(test_file).unwrap();
    assert!(content.contains("Initial content"));
    assert!(content.contains("Appended line"));
    assert!(content.contains("Another line"));
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_delete_file() {
    let test_file = "/tmp/test_delete_file.txt";
    
    // Create test file
    fs::write(test_file, "content to be deleted").unwrap();
    assert!(Path::new(test_file).exists());
    
    // Test delete in REPL
    let code = format!(r#"delete_file("{test_file}")"#);
    let result = eval_in_repl(&code);
    assert!(result.is_ok(), "delete_file should work in REPL: {result:?}");
    
    // Verify file is deleted
    assert!(!Path::new(test_file).exists());
    
    // Test transpiled version
    fs::write(test_file, "content to be deleted again").unwrap();
    assert!(Path::new(test_file).exists());
    
    let code = format!(r#"delete_file("{test_file}")"#);
    let result = eval_transpiled(&code);
    assert!(result.is_ok(), "delete_file should work in transpiler: {result:?}");
    
    // Verify file is deleted
    assert!(!Path::new(test_file).exists());
}

#[test] 
fn test_file_operations_with_existing_functions() {
    let test_file = "/tmp/test_combined_ops.txt";
    
    // Clean up first
    let _ = fs::remove_file(test_file);
    
    // Test combined operations in sequence
    let code = format!(r#"file_exists("{test_file}")"#);
    let result = eval_in_repl(&code).unwrap();
    assert_eq!(result, "false");
    
    let code = format!(r#"write_file("{test_file}", "Hello World\n")"#);
    let result = eval_in_repl(&code);
    assert!(result.is_ok(), "write_file should work: {result:?}");
    
    let code = format!(r#"file_exists("{test_file}")"#);
    let result = eval_in_repl(&code).unwrap();
    assert_eq!(result, "true");
    
    let code = format!(r#"append_file("{test_file}", "Additional line\n")"#);
    let result = eval_in_repl(&code);
    assert!(result.is_ok(), "append_file should work: {result:?}");
    
    // Verify final state
    assert!(Path::new(test_file).exists());
    let content = fs::read_to_string(test_file).unwrap();
    assert!(content.contains("Hello World"));
    assert!(content.contains("Additional line"));
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_error_handling() {
    // Test operations on non-existent files
    
    // Reading non-existent file should error
    let result = eval_in_repl(r#"read_file("/tmp/non_existent_file_12345.txt")"#);
    assert!(result.is_err(), "Reading non-existent file should error");
    
    // Deleting non-existent file should be handled gracefully
    let result = eval_in_repl(r#"delete_file("/tmp/non_existent_file_12345.txt")"#);
    // This might error or return Unit, depending on implementation
    // Just verify it doesn't crash
    let _ = result;
    
    // Appending to non-existent file might create it or error
    let test_file = "/tmp/test_append_nonexistent.txt";
    let _ = fs::remove_file(test_file); // Make sure it doesn't exist
    
    let result = eval_in_repl(&format!(r#"append_file("{test_file}", "test")"#));
    // This behavior depends on implementation - might create file or error
    let _ = result;
    
    // Clean up
    let _ = fs::remove_file(test_file);
}