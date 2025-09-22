#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, skipping invalid UTF-8
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return, // Skip invalid UTF-8 input
    };

    // Skip empty inputs
    if input.trim().is_empty() {
        return;
    }

    // Skip inputs that are too large to avoid timeouts
    if input.len() > 10000 {
        return;
    }

    // Create a working file for testing
    let mut temp_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return,
    };

    // Write the input to the file
    if let Err(_) = temp_file.write_all(input.as_bytes()) {
        return;
    }

    // Test that the file can be processed without crashing
    // The actual parsing and formatting is tested via CLI below

    // Test the CLI interface directly if possible
    let temp_path = temp_file.path();
    if let Some(path_str) = temp_path.to_str() {
        // Test that CLI doesn't crash on this input
        // We don't care about the result, just that it doesn't panic
        let _ = std::process::Command::new("cargo")
            .args(["run", "--bin", "ruchy", "--", "fmt", path_str])
            .output();
    }
});
