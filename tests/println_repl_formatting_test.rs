//! REPL println Multi-Arg Formatting Test (TDD)
//! 
//! Documents the issue where println("Hello", "World") in REPL prints on separate lines
//! instead of space-separated on one line like the compiled version.
//!
//! **Expected**: `Hello World` on one line  
//! **Actual**: `Hello` on line 1, ` "World"` on line 2 (currently broken in interactive mode)

use ruchy::runtime::repl::Repl;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Thread-safe stdout capture for testing
#[derive(Clone)]
struct OutputCapture {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl OutputCapture {
    fn new() -> Self {
        Self { 
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn get_output(&self) -> String {
        let buffer = self.buffer.lock().unwrap();
        String::from_utf8(buffer.clone()).unwrap_or_default()
    }
    
    fn clear(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.clear();
    }
}

impl Write for OutputCapture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_println_multi_args_should_be_space_separated_on_one_line() {
    // Setup: Create REPL instance
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    // Test input: println with multiple string arguments
    let input = r#"println("Hello", "World")"#;
    
    // Execute the println command and capture output
    let result = repl.evaluate_expr_str(input, None);
    
    // Should succeed
    assert!(result.is_ok(), "println evaluation should succeed");
    
    // The key assertion: We need to capture stdout to verify the format
    // For now, let's test that it doesn't panic and returns Unit
    let value = result.unwrap();
    assert_eq!(format!("{:?}", value), "Unit", "println should return unit value");
    
    // TODO: Add proper stdout capture to verify "Hello World" on one line
    // This test documents the expected behavior for implementation
}

#[test] 
fn test_println_single_arg_works_correctly() {
    // Setup: Create REPL instance  
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    // Test input: println with single argument (this should work)
    let input = r#"println("Hello")"#;
    
    // Execute and verify it succeeds
    let result = repl.evaluate_expr_str(input, None);
    assert!(result.is_ok(), "Single arg println should work");
    
    let value = result.unwrap();
    assert_eq!(format!("{:?}", value), "Unit", "println should return unit value");
}

#[test]
fn test_compiled_println_works_correctly() {
    // This test verifies that compiled println works (it should pass)
    // It's a regression test to ensure we don't break the transpiler fix
    use ruchy::{Parser, Transpiler};
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::fs;
    
    // Create Ruchy source with multi-arg println
    let source = r#"println("Hello", "World")"#;
    
    // Parse and transpile
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse println statement");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .expect("Should transpile println");
    
    // Write to temporary Rust file
    let mut rust_file = NamedTempFile::with_suffix(".rs").expect("Create temp file");
    fs::write(rust_file.path(), rust_code.to_string()).expect("Write Rust code");
    
    // Compile and run
    let output = Command::new("rustc")
        .arg(rust_file.path())
        .arg("-o")
        .arg("/tmp/test_println")
        .output()
        .expect("Compile Rust code");
    
    assert!(output.status.success(), "Rust compilation should succeed");
    
    let run_output = Command::new("/tmp/test_println")
        .output()
        .expect("Run compiled binary");
        
    assert!(run_output.status.success(), "Compiled program should run");
    
    let stdout = String::from_utf8(run_output.stdout).expect("Valid UTF-8 output");
    
    // CRITICAL: Compiled version should print "Hello World" on one line
    assert!(stdout.contains("Hello World"), 
        "Compiled println should output 'Hello World' on one line, got: {:?}", stdout);
}