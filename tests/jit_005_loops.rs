//! JIT-005: Loop Support (while, for)
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Extend JIT compiler with iterative control flow
//! Target: Iterative algorithms 100-1000x faster than AST interpreter
//!
//! Test Strategy:
//! 1. While loops - basic iteration with condition
//! 2. While loops - with break statement
//! 3. While loops - accumulator patterns (sum, product)
//! 4. For loops - range iteration (1..n)
//! 5. For loops - with break
//! 6. Nested loops - 2D iteration patterns
//! 7. Loop-based algorithms - factorial, fibonacci, sum
//! 8. Performance validation - iterative vs recursive

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: While Loops (Basic)
// ============================================================================

#[test]
fn test_jit_005_while_simple_countdown() {
    // Test: Count down from 5 to 0
    let code = r"
        let mut x = 5;
        while x > 0 {
            x = x - 1;
        }
        x
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile while loop: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "x should be 0 after countdown");
}

#[test]
fn test_jit_005_while_accumulator_sum() {
    // Test: Sum numbers 1 to 10 using while loop
    let code = r"
        let mut i = 1;
        let mut sum = 0;
        while i <= 10 {
            sum = sum + i;
            i = i + 1;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile while sum: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "Sum 1..10 should be 55");
}

#[test]
fn test_jit_005_while_with_if() {
    // Test: While loop with conditional logic inside
    let code = r"
        let mut x = 0;
        let mut count = 0;
        while x < 10 {
            if x % 2 == 0 {
                count = count + 1;
            }
            x = x + 1;
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile while with if: {:?}", result.err());
    assert_eq!(result.unwrap(), 5, "Should count 5 even numbers (0,2,4,6,8)");
}

// ============================================================================
// RED-002: While Loops with Break
// ============================================================================

#[test]
fn test_jit_005_while_break_simple() {
    // Test: Break out of loop when condition met
    let code = r"
        let mut x = 0;
        while x < 100 {
            if x == 5 {
                break;
            }
            x = x + 1;
        }
        x
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile while with break: {:?}", result.err());
    assert_eq!(result.unwrap(), 5, "Should break at x=5");
}

#[test]
fn test_jit_005_while_break_search() {
    // Test: Search for first multiple of 7 greater than 50
    let code = r"
        let mut x = 51;
        while x < 100 {
            if x % 7 == 0 {
                break;
            }
            x = x + 1;
        }
        x
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile while break search: {:?}", result.err());
    assert_eq!(result.unwrap(), 56, "First multiple of 7 > 50 is 56");
}

// ============================================================================
// RED-003: For Loops (Range Iteration)
// ============================================================================

#[test]
fn test_jit_005_for_range_simple() {
    // Test: Sum numbers using for loop
    let code = r"
        let mut sum = 0;
        for i in 1..11 {
            sum = sum + i;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile for loop: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "Sum 1..10 should be 55");
}

#[test]
fn test_jit_005_for_range_inclusive() {
    // Test: Inclusive range (1..=10)
    let code = r"
        let mut sum = 0;
        for i in 1..=10 {
            sum = sum + i;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile for inclusive: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "Sum 1..=10 should be 55");
}

#[test]
fn test_jit_005_for_range_with_if() {
    // Test: For loop with conditional (count evens)
    let code = r"
        let mut count = 0;
        for i in 0..10 {
            if i % 2 == 0 {
                count = count + 1;
            }
        }
        count
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile for with if: {:?}", result.err());
    assert_eq!(result.unwrap(), 5, "Count evens in 0..10 should be 5");
}

// ============================================================================
// RED-004: For Loops with Break
// ============================================================================

#[test]
fn test_jit_005_for_break_search() {
    // Test: Break when first even number > 5 found
    let code = r"
        let mut found = 0;
        for i in 0..20 {
            if i > 5 && i % 2 == 0 {
                found = i;
                break;
            }
        }
        found
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile for with break: {:?}", result.err());
    assert_eq!(result.unwrap(), 6, "First even > 5 is 6");
}

// ============================================================================
// RED-005: Nested Loops
// ============================================================================

#[test]
fn test_jit_005_nested_loops_multiplication_table() {
    // Test: 5x5 multiplication table sum
    let code = r"
        let mut sum = 0;
        for i in 1..6 {
            for j in 1..6 {
                sum = sum + (i * j);
            }
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile nested loops: {:?}", result.err());
    // Sum of 5x5 multiplication table: each row sums to i*(1+2+3+4+5)=i*15
    // Total: 1*15 + 2*15 + 3*15 + 4*15 + 5*15 = 15*(1+2+3+4+5) = 15*15 = 225
    assert_eq!(result.unwrap(), 225, "5x5 multiplication table sum");
}

#[test]
fn test_jit_005_nested_while_loops() {
    // Test: Nested while loops
    let code = r"
        let mut i = 0;
        let mut sum = 0;
        while i < 5 {
            let mut j = 0;
            while j < 5 {
                sum = sum + 1;
                j = j + 1;
            }
            i = i + 1;
        }
        sum
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile nested while: {:?}", result.err());
    assert_eq!(result.unwrap(), 25, "5x5 iterations = 25");
}

// ============================================================================
// RED-006: Loop-Based Algorithms
// ============================================================================

#[test]
fn test_jit_005_factorial_iterative() {
    // Test: Factorial using loop (5! = 120)
    let code = r"
        fun factorial(n: i32) -> i32 {
            let mut result = 1;
            let mut i = 1;
            while i <= n {
                result = result * i;
                i = i + 1;
            }
            result
        }
        factorial(5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile factorial: {:?}", result.err());
    assert_eq!(result.unwrap(), 120, "5! = 120");
}

#[test]
fn test_jit_005_fibonacci_iterative() {
    // Test: Fibonacci using loop (fib(10) = 55)
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                let mut a = 0;
                let mut b = 1;
                let mut i = 2;
                while i <= n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                    i = i + 1;
                }
                b
            }
        }
        fib(10)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile fibonacci: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "fib(10) = 55");
}

#[test]
fn test_jit_005_sum_range_function() {
    // Test: Sum range using for loop
    let code = r"
        fun sum_range(start: i32, end: i32) -> i32 {
            let mut sum = 0;
            for i in start..end {
                sum = sum + i;
            }
            sum
        }
        sum_range(1, 11)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile sum_range: {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "sum_range(1, 11) = 55");
}

// ============================================================================
// RED-007: Performance Validation
// ============================================================================

#[test]
fn test_jit_005_performance_iterative_sum() {
    // Performance: Sum 1..1000 should be very fast with JIT
    let code = r"
        fun sum_to_n(n: i32) -> i32 {
            let mut sum = 0;
            let mut i = 1;
            while i <= n {
                sum = sum + i;
                i = i + 1;
            }
            sum
        }
        sum_to_n(1000)
    ";

    let ast = Parser::new(code).parse().unwrap();

    // Warmup
    {
        let mut compiler = JitCompiler::new().unwrap();
        let warmup_result = compiler.compile_and_execute(&ast);
        assert_eq!(warmup_result.unwrap(), 500500, "sum(1..1000) = 500500");
    }

    // Benchmark: 100 iterations
    let iterations = 100;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert_eq!(result.unwrap(), 500500);
    }
    let total_elapsed = start.elapsed();
    let avg_elapsed = total_elapsed / iterations;

    println!("\n=== JIT-005 Performance Benchmark ===");
    println!("sum_to_n(1000) JIT avg over {iterations} runs: {avg_elapsed:?}");
    println!("Target: <1ms per run (includes compilation overhead)");

    // Performance assertion: Should be faster than 1ms
    let avg_micros = avg_elapsed.as_micros();
    assert!(
        avg_micros < 1000,
        "JIT should be faster than 1ms. Actual: {avg_micros}µs"
    );

    println!("Actual: {avg_micros}µs - ✅ Performance target met");
}
