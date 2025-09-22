#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 strings
    if let Ok(input) = std::str::from_utf8(data) {
        // Focus on module path patterns
        let patterns = [
            format!("fn test(x: {}) {{ x }}", input),
            format!("{}::function()", input),
            format!("use {}", input),
            format!("let x: {} = y", input),
            format!("Result::{}", input),
            format!("Option::{}", input),
        ];

        for pattern in &patterns {
            // Limit input size to avoid OOM
            if pattern.len() > 10000 {
                continue;
            }

            let mut parser = Parser::new(pattern);
            // We don't care if it fails, just that it doesn't panic
            let _ = parser.parse();
        }
    }
});
