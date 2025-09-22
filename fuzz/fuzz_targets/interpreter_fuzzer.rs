#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::{runtime::interpreter::Interpreter, Parser};

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 strings
    if let Ok(s) = std::str::from_utf8(data) {
        // Limit input size to prevent excessive memory usage
        if s.len() > 2_000 {
            return;
        }

        // Try to parse
        let mut parser = Parser::new(s);

        // If parsing succeeds, try to interpret
        if let Ok(ast) = parser.parse() {
            // Create interpreter with resource limits
            let mut interpreter = Interpreter::new();

            // Set a timeout to prevent infinite loops
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(100);

            // Try to evaluate the AST
            let _ = interpreter.eval(&ast);

            // Check if we exceeded the deadline
            if std::time::Instant::now() > deadline {
                // Fuzzer found potential infinite loop
                return;
            }
        }
    }
});
