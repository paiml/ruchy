//! Resource usage verification tests

use ruchy::frontend::parser::Parser;
use std::time::{Duration, Instant};

const MAX_MEMORY_MB: usize = 100;
const MAX_TIME_MS: u128 = 1000;
const MAX_DEPTH: u32 = 4;

#[test]
fn verify_memory_usage() {
    let before = get_memory_usage();
    
    // Run a typical heavy test scenario
    for _ in 0..100 {
        let expr = generate_test_expr(MAX_DEPTH);
        let source = format!("{expr:?}");
        let _ = Parser::new(&source).parse();
    }
    
    let after = get_memory_usage();
    let delta_mb = (after.saturating_sub(before)) / 1_048_576;
    
    assert!(
        delta_mb < MAX_MEMORY_MB,
        "Test uses {delta_mb}MB (limit: {MAX_MEMORY_MB}MB)"
    );
}

#[test]
fn verify_generation_time() {
    let start = Instant::now();
    
    // Generate expressions with bounded depth
    for depth in 0..=MAX_DEPTH {
        for _ in 0..10 {
            let _ = generate_test_expr(depth);
        }
    }
    
    let elapsed = start.elapsed().as_millis();
    assert!(
        elapsed < MAX_TIME_MS,
        "Generation took {elapsed}ms (limit: {MAX_TIME_MS}ms)"
    );
}

#[test]
fn verify_parser_memory_bounded() {
    // Test that parser doesn't consume excessive memory on pathological input
    let long_identifier = "x".repeat(1000);
    let many_expressions = (0..100).fold(String::new(), |mut acc, i| {
        use std::fmt::Write;
        let _ = write!(acc, "let x{i} = {i}; ");
        acc
    });
    
    let pathological_inputs = vec![
        // Deeply nested parentheses
        "((((((((((((((((((((1))))))))))))))))))))",
        // Long identifier
        &long_identifier,
        // Many small expressions
        &many_expressions,
    ];
    
    for input in pathological_inputs {
        let before = get_memory_usage();
        let _ = Parser::new(input).parse();
        let after = get_memory_usage();
        
        let delta_mb = (after.saturating_sub(before)) / 1_048_576;
        assert!(
            delta_mb < 10,
            "Parser used {delta_mb}MB on pathological input"
        );
    }
}

#[test]
#[ignore = "Run with: cargo test -- --ignored"]
fn stress_test_with_memory_limit() {
    // Heavy stress test - only run when explicitly requested
    let start = Instant::now();
    let initial_memory = get_memory_usage();
    
    for _ in 0..1000 {
        let _ = generate_test_expr(6); // Deep expression
        
        // Check memory periodically
        if start.elapsed() > Duration::from_millis(100) {
            let current_memory = get_memory_usage();
            let delta_mb = (current_memory.saturating_sub(initial_memory)) / 1_048_576;
            
            assert!(delta_mb <= 500, "Memory usage exceeded 500MB during stress test");
        }
    }
}

fn get_memory_usage() -> usize {
    // Use /proc/self/status on Linux for memory info
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    // Fallback: use a rough estimate based on allocation
    #[cfg(not(target_os = "linux"))]
    {
        // This is a very rough approximation
        // In production, use proper memory profiling tools
        1_000_000 // Return 1MB as default
    }
    
    0
}

fn generate_test_expr(depth: u32) -> String {
    // Simple expression generator for testing
    if depth == 0 {
        "42".to_string()
    } else {
        format!("({} + {})", generate_test_expr(depth - 1), generate_test_expr(depth - 1))
    }
}

