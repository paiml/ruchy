#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::parser::Parser;
use ruchy::transpiler::Transpiler;
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Very limited input size for full pipeline
        if s.len() > 1_000 {
            return;
        }
        
        // Parse
        let mut parser = Parser::new(s);
        if let Ok(ast) = parser.parse_module() {
            // Transpile
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile_module(&ast) {
                // Try to compile (but limit this to avoid resource exhaustion)
                static mut COMPILE_COUNT: u32 = 0;
                unsafe {
                    COMPILE_COUNT += 1;
                    if COMPILE_COUNT % 100 != 0 {
                        return; // Only compile every 100th input
                    }
                }
                
                // Write to working file
                if let Ok(mut temp_file) = NamedTempFile::new() {
                    if temp_file.write_all(rust_code.as_bytes()).is_ok() {
                        if temp_file.flush().is_ok() {
                            // Try compilation (with timeout)
                            let output_binary = temp_file.path().with_extension("exe");
                            let _ = Command::new("timeout")
                                .arg("1") // 1 second timeout
                                .arg("rustc")
                                .arg("--edition=2021")
                                .arg("-O0") // No optimization for speed
                                .arg("-o")
                                .arg(&output_binary)
                                .arg(temp_file.path())
                                .output();
                            
                            // Clean up
                            if output_binary.exists() {
                                std::fs::remove_file(output_binary).ok();
                            }
                        }
                    }
                }
            }
        }
    }
});