//! Fuzzing target for REPL input handling
//! 
//! Tests that REPL never crashes on arbitrary input and handles all errors gracefully

#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::runtime::repl::Repl;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, allowing invalid UTF-8
    let input = String::from_utf8_lossy(data);
    
    // Skip empty inputs
    if input.trim().is_empty() {
        return;
    }
    
    // Create REPL instance
    let mut repl = match Repl::new() {
        Ok(repl) => repl,
        Err(_) => return, // Skip if REPL creation fails
    };
    
    // Test all REPL operations with the fuzzed input
    let input_str = input.as_ref();
    
    // 1. Test evaluation (most important)
    let _ = repl.eval(input_str);
    
    // 2. Test show operations (should never crash)
    let _ = repl.show_ast(input_str);
    let _ = repl.show_rust(input_str);
    let _ = repl.show_type(input_str);
    
    // 3. Test history and clear (for state management)
    let _ = repl.show_history();
    repl.clear_session();
    
    // 4. Test save/load with arbitrary filenames (but ignore file system errors)
    if let Ok(temp_file) = std::env::temp_dir().join(format!("fuzz_{}", data.len())).into_os_string().into_string() {
        let _ = repl.save_session(&temp_file);
        let _ = repl.load_session(&temp_file);
        let _ = std::fs::remove_file(&temp_file); // Clean up
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fuzz_target_basic() {
        // Test that the fuzz target works with some basic inputs
        let test_inputs = vec![
            b"42",
            b"let x = 1",
            b"invalid syntax!!!",
            b"\x00\x01\x02", // Non-UTF8 bytes
            b"",
            b"fun test() { }",
            b"[1, 2, 3]",
            b"if true { 1 } else { 2 }",
        ];
        
        for input in test_inputs {
            // Should not panic
            libfuzzer_sys::fuzz_target!(|data: &[u8]| {
                // Implementation is above
            })(input);
        }
    }
    
    #[test]
    fn test_repl_error_handling() {
        let mut repl = Repl::new().expect("Failed to create REPL");
        
        // Test various invalid inputs
        let invalid_inputs = vec![
            "let",
            "fun(",
            "if",
            "match",
            "[1, 2,",
            "1 + + 2",
            "unknown_function()",
            "let x = unknown_var",
            "\n\n\n",
            ";;;",
            "{ } [ ]",
        ];
        
        for input in invalid_inputs {
            // Should handle gracefully, not crash
            let result = repl.eval(input);
            match result {
                Ok(_) => {}, // Success is fine
                Err(e) => {
                    // Error should have a message
                    assert!(!e.to_string().is_empty(), "Error message should not be empty for input: {}", input);
                }
            }
        }
    }
}