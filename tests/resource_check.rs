//! Resource usage verification tests

use ruchy::frontend::parser::Parser;
use ruchy::testing::generators::{arb_expr_with_depth, AstGenConfig};
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
        "Test uses {}MB (limit: {}MB)",
        delta_mb,
        MAX_MEMORY_MB
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
        "Generation took {}ms (limit: {}ms)",
        elapsed,
        MAX_TIME_MS
    );
}

#[test]
fn verify_parser_memory_bounded() {
    // Test that parser doesn't consume excessive memory on pathological input
    let pathological_inputs = vec![
        // Deeply nested parentheses
        "((((((((((((((((((((1))))))))))))))))))))",
        // Long identifier
        &"x".repeat(1000),
        // Many small expressions
        &(0..100).map(|i| format!("let x{i} = {i}; ")).collect::<String>(),
    ];
    
    for input in pathological_inputs {
        let before = get_memory_usage();
        let _ = Parser::new(input).parse();
        let after = get_memory_usage();
        
        let delta_mb = (after.saturating_sub(before)) / 1_048_576;
        assert!(
            delta_mb < 10,
            "Parser used {}MB on pathological input",
            delta_mb
        );
    }
}

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn stress_test_with_memory_limit() {
    // Heavy stress test - only run when explicitly requested
    let config = AstGenConfig {
        max_depth: 6,
        max_list_size: 20,
        max_identifier_len: 50,
        favor_well_typed: false,
    };
    
    let start = Instant::now();
    let initial_memory = get_memory_usage();
    
    for _ in 0..1000 {
        let _ = generate_test_expr_with_config(&config);
        
        // Check memory periodically
        if start.elapsed() > Duration::from_millis(100) {
            let current_memory = get_memory_usage();
            let delta_mb = (current_memory.saturating_sub(initial_memory)) / 1_048_576;
            
            if delta_mb > 500 {
                panic!("Memory usage exceeded 500MB during stress test");
            }
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

fn generate_test_expr_with_config(_config: &AstGenConfig) -> String {
    // Placeholder for config-based generation
    // In real implementation, use the generators module
    generate_test_expr(3)
}