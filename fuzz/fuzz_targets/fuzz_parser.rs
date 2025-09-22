#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        let mut parser = Parser::new(input);
        // Parser should never panic, even on invalid input
        let _ = parser.parse();
    }
});
