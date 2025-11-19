//! JIT-005B: Continue Statement (Complete Loop Control Flow)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Add continue statement to complement break (JIT-005)
//! Target: Complete loop control flow for iterative algorithms
//!
//! Test Strategy:
//! 1. Continue in while loops - skip to next iteration
//! 2. Continue in for loops - skip to increment
//! 3. Continue with conditions - selective iteration
//! 4. Continue in nested loops - innermost only
//! 5. Continue vs break - verify different behaviors
//! 6. Continue with accumulator - skip specific values

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Continue in While Loops
// ============================================================================

#[test]
fn test_jit_005b_while_continue_simple() {
    // Test: Skip odd numbers, count only evens
    let code = r"
        let mut x = 0;
        let mut count = 0;
        while x < 10 {
            x = x + 1;
            if x % 2 == 1 {
                continue;
            }
            count = count + 1;
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile while with continue: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        5,
        "Should count 5 even numbers (2,4,6,8,10)"
    );
}

#[test]
fn test_jit_005b_while_continue_accumulator() {
    // Test: Sum only multiples of 3
    let code = r"
        let mut i = 0;
        let mut sum = 0;
        while i < 10 {
            i = i + 1;
            if i % 3 != 0 {
                continue;
            }
            sum = sum + i;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile while continue sum: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 18, "Sum of 3,6,9 should be 18");
}

#[test]
fn test_jit_005b_while_continue_multiple_conditions() {
    // Test: Skip both multiples of 2 and 3
    let code = r"
        let mut i = 0;
        let mut count = 0;
        while i < 20 {
            i = i + 1;
            if i % 2 == 0 {
                continue;
            }
            if i % 3 == 0 {
                continue;
            }
            count = count + 1;
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile multiple continues: {:?}",
        result.err()
    );
    // Numbers 1-20 that are NOT divisible by 2 or 3:
    // 1,5,7,11,13,17,19 = 7 numbers
    assert_eq!(
        result.unwrap(),
        7,
        "Should count 7 numbers not divisible by 2 or 3"
    );
}

// ============================================================================
// RED-002: Continue in For Loops
// ============================================================================

#[test]
fn test_jit_005b_for_continue_simple() {
    // Test: Sum only odd numbers using for loop
    let code = r"
        let mut sum = 0;
        for i in 0..10 {
            if i % 2 == 0 {
                continue;
            }
            sum = sum + i;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile for with continue: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 25, "Sum of 1,3,5,7,9 should be 25");
}

#[test]
fn test_jit_005b_for_continue_range_inclusive() {
    // Test: Continue in inclusive range
    let code = r"
        let mut count = 0;
        for i in 1..=10 {
            if i <= 5 {
                continue;
            }
            count = count + 1;
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile for inclusive with continue: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "Should count 5 numbers (6,7,8,9,10)");
}

#[test]
fn test_jit_005b_for_continue_with_computation() {
    // Test: Skip negative results
    let code = r"
        let mut sum = 0;
        for i in 0..10 {
            let val = i - 5;
            if val < 0 {
                continue;
            }
            sum = sum + val;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile for continue with computation: {:?}",
        result.err()
    );
    // val: 0-5=-5(skip), 1-5=-4(skip), ..., 5-5=0, 6-5=1, 7-5=2, 8-5=3, 9-5=4
    // sum = 0+1+2+3+4 = 10
    assert_eq!(result.unwrap(), 10, "Sum should be 10");
}

// ============================================================================
// RED-003: Continue vs Break
// ============================================================================

#[test]
fn test_jit_005b_continue_vs_break() {
    // Test: Continue skips iteration, break exits loop
    let code = r"
        let mut continue_count = 0;
        for i in 0..10 {
            if i == 5 {
                continue;
            }
            continue_count = continue_count + 1;
        }

        let mut break_count = 0;
        for i in 0..10 {
            if i == 5 {
                break;
            }
            break_count = break_count + 1;
        }

        continue_count - break_count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile continue vs break: {:?}",
        result.err()
    );
    // continue: 9 iterations (skips i=5)
    // break: 5 iterations (stops at i=5)
    // difference: 9-5 = 4
    assert_eq!(
        result.unwrap(),
        4,
        "Continue=9 iterations, break=5 iterations, diff=4"
    );
}

// ============================================================================
// RED-004: Continue in Nested Loops
// ============================================================================

#[test]
fn test_jit_005b_nested_loops_continue_inner() {
    // Test: Continue in inner loop only
    let code = r"
        let mut count = 0;
        for i in 0..3 {
            for j in 0..5 {
                if j == 2 {
                    continue;
                }
                count = count + 1;
            }
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile nested continue: {:?}",
        result.err()
    );
    // Outer: 3 iterations, Inner: 5 iterations (skip j=2)
    // 3 * 4 = 12
    assert_eq!(result.unwrap(), 12, "3 outer * 4 inner (skip j=2) = 12");
}

#[test]
fn test_jit_005b_nested_loops_continue_outer() {
    // Test: Continue can skip inner loop entirely
    let code = r"
        let mut count = 0;
        let mut i = 0;
        while i < 3 {
            i = i + 1;
            if i == 2 {
                continue;
            }
            let mut j = 0;
            while j < 4 {
                j = j + 1;
                count = count + 1;
            }
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile outer loop continue: {:?}",
        result.err()
    );
    // i=1: inner loop runs 4 times
    // i=2: continue skips inner loop
    // i=3: inner loop runs 4 times
    // total: 4+0+4 = 8
    assert_eq!(result.unwrap(), 8, "Skip middle iteration: 4+0+4 = 8");
}

// ============================================================================
// RED-005: Continue with Algorithm
// ============================================================================

#[test]
fn test_jit_005b_filter_and_transform() {
    // Test: Filter then transform pattern
    let code = r"
        fun process_range(start: i32, end: i32) -> i32 {
            let mut result = 0;
            for i in start..end {
                // Skip numbers divisible by 5
                if i % 5 == 0 {
                    continue;
                }
                // Double non-multiples of 5
                result = result + (i * 2);
            }
            result
        }
        process_range(0, 10)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile filter-transform: {:?}",
        result.err()
    );
    // Numbers 0-9, skip 0 and 5, double rest: (1+2+3+4+6+7+8+9)*2 = 40*2 = 80
    assert_eq!(
        result.unwrap(),
        80,
        "Filter and double: (1+2+3+4+6+7+8+9)*2 = 80"
    );
}

#[test]
fn test_jit_005b_prime_sieve_pattern() {
    // Test: Simple primality check pattern with continue
    let code = r"
        let mut prime_count = 0;
        for n in 2..20 {
            let mut is_prime = 1;
            let mut i = 2;
            while i * i <= n {
                if n % i == 0 {
                    is_prime = 0;
                    break;
                }
                i = i + 1;
            }
            if is_prime == 0 {
                continue;
            }
            prime_count = prime_count + 1;
        }
        prime_count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile prime sieve: {:?}",
        result.err()
    );
    // Primes 2-19: 2,3,5,7,11,13,17,19 = 8 primes
    assert_eq!(result.unwrap(), 8, "8 primes in range 2-19");
}
