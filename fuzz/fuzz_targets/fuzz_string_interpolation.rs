#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::Parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test that parsing strings with potential interpolation never panics
        let mut parser = Parser::new(s);
        let _ = parser.parse();

        // Also test string interpolation parsing directly if it looks like a string
        if s.starts_with('"') && s.ends_with('"') && s.len() > 2 {
            let mut parser = Parser::new(s);
            let _ = parser.parse();
        }
    }
});
