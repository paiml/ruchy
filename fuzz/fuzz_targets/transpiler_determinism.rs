//! Fuzz test for transpiler determinism
//! 
//! Property: The transpiler must produce identical output for identical input,
//! regardless of environmental conditions or build configuration.

#![no_main]
use libfuzzer_sys::fuzz_target;
use ruchy::{Parser, Transpiler};
use std::collections::HashMap;
use std::env;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to a string
    if let Ok(input) = std::str::from_utf8(data) {
        // Skip if input is too large (to avoid DoS)
        if input.len() > 10000 {
            return;
        }
        
        // Test determinism by transpiling the same input multiple times
        let mut results = Vec::new();
        
        for seed in 0..3 {
            // Perturb environment to test resilience
            perturb_environment(seed);
            
            // Try to parse
            let mut parser = Parser::new(input);
            if let Ok(ast) = parser.parse() {
                // Transpile
                let transpiler = Transpiler::new();
                if let Ok(tokens) = transpiler.transpile(&ast) {
                    let output = tokens.to_string();
                    results.push(output);
                }
            }
        }
        
        // All results should be identical (deterministic)
        if results.len() > 1 {
            let first = &results[0];
            for result in &results[1..] {
                assert_eq!(
                    first, result,
                    "Non-deterministic transpilation for input: {:?}",
                    input
                );
            }
        }
    }
});

/// Perturb the environment in deterministic but unusual ways
fn perturb_environment(seed: u64) {
    // Change hash seed (if Rust uses it)
    env::set_var("RUST_HASH_SEED", seed.to_string());
    
    // Could add more perturbations here:
    // - Thread pool size changes
    // - Memory allocation patterns
    // - etc.
}