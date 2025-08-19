#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::parser::Parser;

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 strings
    if let Ok(s) = std::str::from_utf8(data) {
        // Limit input size to prevent excessive memory usage
        if s.len() > 10_000 {
            return;
        }
        
        // Create parser and try to parse
        let mut parser = Parser::new(s);
        
        // Try parsing as module
        let _ = parser.parse_module();
        
        // Try parsing as expression
        let mut parser2 = Parser::new(s);
        let _ = parser2.parse_expression();
        
        // Try parsing as statement
        let mut parser3 = Parser::new(s);
        let _ = parser3.parse_statement();
    }
});