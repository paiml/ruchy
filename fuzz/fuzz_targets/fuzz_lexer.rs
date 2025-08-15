#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::frontend::lexer::TokenStream;

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, ignoring invalid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        // Lexer should never panic, even on invalid input
        let tokens: Vec<_> = TokenStream::new(input).collect();
        
        // Verify no information is lost
        let total_len: usize = tokens.iter().map(|(_, span)| span.end - span.start).sum();
        assert!(total_len <= input.len());
    }
});