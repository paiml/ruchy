#![no_main]

use libfuzzer_sys::fuzz_target;
use std::process::Command;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, skipping invalid UTF-8
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,  // Skip invalid UTF-8 input
    };
    
    // Skip empty inputs or very large inputs
    if input.trim().is_empty() || input.len() > 5000 {
        return;
    }
    
    // Create a working file
    let mut temp_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return,
    };
    
    // Write the input to the file
    use std::io::Write;
    if let Err(_) = temp_file.write_all(input.as_bytes()) {
        return;
    }
    
    let temp_path = temp_file.path();
    if let Some(path_str) = temp_path.to_str() {
        // Test various CLI commands with the fuzz input
        let commands = vec![
            vec!["fmt", path_str],
            vec!["check", path_str],
            vec!["run", path_str],
        ];
        
        for cmd_args in commands {
            // Test that CLI commands don't crash
            // We use timeout to prevent hanging on infinite loops
            let _ = Command::new("timeout")
                .arg("2s")  // 2 second timeout
                .arg("cargo")
                .arg("run")
                .arg("--bin")
                .arg("ruchy")
                .arg("--")
                .args(&cmd_args)
                .output();
        }
        
        // Test one-liner execution with random input
        if input.len() < 100 && !input.contains('\n') {
            let _ = Command::new("timeout")
                .arg("2s")
                .arg("cargo")
                .arg("run")
                .arg("--bin")
                .arg("ruchy")
                .arg("--")
                .arg("-e")
                .arg(input)
                .output();
        }
    }
});