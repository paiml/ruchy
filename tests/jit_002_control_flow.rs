//! JIT-002: Control Flow and Function Calls
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Implement control flow (if/else) and function calls to enable fibonacci recursion
//! Target: fibonacci(20) <0.5ms (vs 19ms AST interpreter - 50-100x speedup)
//!
//! Test Strategy:
//! 1. Simple if/else (no recursion)
//! 2. Variables and comparisons
//! 3. Simple function calls (no recursion)
//! 4. Recursive function calls (fibonacci)
//! 5. Performance validation

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Simple If/Else (No Variables)
// ============================================================================

#[test]
fn test_jit_002_if_true_literal() {
    // Test: if true { 42 } else { 0 }
    let code = r"
        if true {
            42
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile if/else: {:?}", result.err());
    assert_eq!(result.unwrap(), 42, "Should take true branch");
}

#[test]
fn test_jit_002_if_false_literal() {
    // Test: if false { 42 } else { 99 }
    let code = r"
        if false {
            42
        } else {
            99
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile if/else: {:?}", result.err());
    assert_eq!(result.unwrap(), 99, "Should take false branch");
}

// ============================================================================
// RED-002: Comparisons (<=, <, >, >=, ==, !=)
// ============================================================================

#[test]
fn test_jit_002_comparison_less_equal() {
    // Test: if 5 <= 10 { 1 } else { 0 }
    let code = r"
        if 5 <= 10 {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile <= comparison: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "5 <= 10 should be true");
}

#[test]
fn test_jit_002_comparison_equal() {
    // Test: if 42 == 42 { 1 } else { 0 }
    let code = r"
        if 42 == 42 {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile == comparison: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "42 == 42 should be true");
}

// ============================================================================
// RED-003: Variables in If Conditions
// ============================================================================

#[test]
fn test_jit_002_if_with_variable() {
    // Test: let n = 5; if n <= 1 { n } else { 99 }
    let code = r"
        let n = 5;
        if n <= 1 {
            n
        } else {
            99
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile variable in if: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 99, "n=5 not <= 1, should take else branch");
}

#[test]
fn test_jit_002_if_with_variable_true_branch() {
    // Test: let n = 1; if n <= 1 { n } else { 99 }
    let code = r"
        let n = 1;
        if n <= 1 {
            n
        } else {
            99
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile variable in if: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "n=1 <= 1, should return n");
}

// ============================================================================
// RED-004: Simple Function Calls (No Recursion)
// ============================================================================

#[test]
fn test_jit_002_simple_function_call() {
    // Test: fun add(x, y) { x + y }; add(2, 3)
    let code = r"
        fun add(x: i32, y: i32) -> i32 {
            x + y
        }
        add(2, 3)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile function call: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "add(2, 3) should return 5");
}

#[test]
fn test_jit_002_function_with_if() {
    // Test: fun max(x, y) { if x > y { x } else { y } }; max(10, 5)
    let code = r"
        fun max(x: i32, y: i32) -> i32 {
            if x > y {
                x
            } else {
                y
            }
        }
        max(10, 5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile function with if: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 10, "max(10, 5) should return 10");
}

// ============================================================================
// RED-005: Recursive Functions (Fibonacci)
// ============================================================================

#[test]
fn test_jit_002_fibonacci_base_case_0() {
    // Test: fib(0) should return 0
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(0)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile fib(0): {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "fib(0) should return 0");
}

#[test]
fn test_jit_002_fibonacci_base_case_1() {
    // Test: fib(1) should return 1
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(1)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile fib(1): {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "fib(1) should return 1");
}

#[test]
fn test_jit_002_fibonacci_small() {
    // Test: fib(5) should return 5 (fibonacci sequence: 0, 1, 1, 2, 3, 5)
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(5)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile fib(5): {:?}", result.err());
    assert_eq!(result.unwrap(), 5, "fib(5) should return 5");
}

#[test]
fn test_jit_002_fibonacci_10() {
    // Test: fib(10) should return 55
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(10)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile fib(10): {:?}", result.err());
    assert_eq!(result.unwrap(), 55, "fib(10) should return 55");
}

#[test]
fn test_jit_002_fibonacci_20() {
    // Test: fib(20) should return 6765
    // This is the main performance validation test
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(20)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();

    // Measure compile + execute time
    let start = std::time::Instant::now();
    let result = compiler.compile_and_execute(&ast);
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Should compile fib(20): {:?}", result.err());
    assert_eq!(result.unwrap(), 6765, "fib(20) should return 6765");

    // Performance target: <0.5ms for fib(20) vs 19ms AST interpreter
    // Note: First iteration may be slower due to JIT warmup
    println!("JIT fib(20) execution time: {elapsed:?}");

    // Relaxed assertion for first run - we'll measure properly in benchmarks
    assert!(
        elapsed.as_millis() < 100,
        "JIT should be faster than 100ms (AST: 19ms)"
    );
}

// ============================================================================
// RED-006: Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_jit_002_nested_if() {
    // Test: Nested if/else
    let code = r"
        let x = 5;
        if x > 10 {
            1
        } else {
            if x > 3 {
                2
            } else {
                3
            }
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile nested if: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 2, "x=5 > 3, should return 2");
}

#[test]
fn test_jit_002_multiple_functions() {
    // Test: Multiple function definitions
    let code = r"
        fun double(x: i32) -> i32 {
            x * 2
        }
        fun triple(x: i32) -> i32 {
            x * 3
        }
        double(5) + triple(3)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile multiple functions: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 19, "double(5) + triple(3) = 10 + 9 = 19");
}

// ============================================================================
// PERFORMANCE: JIT vs AST Interpreter Benchmark
// ============================================================================

#[test]
fn test_jit_002_performance_benchmark_fibonacci() {
    // Performance validation: JIT should be 50-100x faster than AST interpreter
    // Target: fibonacci(20) <0.5ms vs 19ms AST interpreter
    let code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        fib(20)
    ";

    let ast = Parser::new(code).parse().unwrap();

    // Warmup: First run to ensure JIT libraries are loaded
    {
        let mut compiler = JitCompiler::new().unwrap();
        let warmup_result = compiler.compile_and_execute(&ast);
        assert_eq!(
            warmup_result.unwrap(),
            6765,
            "Warmup: fib(20) should return 6765"
        );
    }

    // Benchmark: Run 100 iterations (compile + execute)
    // Note: Each iteration creates fresh compiler to avoid "duplicate definition" errors
    // This measures TOTAL JIT overhead (compilation + execution)
    let iterations = 100;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert_eq!(result.unwrap(), 6765);
    }
    let total_elapsed = start.elapsed();
    let avg_elapsed = total_elapsed / iterations;

    println!("\n=== JIT-002 Performance Benchmark ===");
    println!("fibonacci(20) JIT avg over {iterations} runs: {avg_elapsed:?}");
    println!("fibonacci(20) JIT total time: {total_elapsed:?}");
    println!("Target: <500µs per run (0.5ms)");
    println!("Baseline: 19ms AST interpreter");

    // Performance assertion: Should be significantly faster than AST interpreter
    // Note: First implementation may not hit <0.5ms target, but should be <19ms
    let avg_micros = avg_elapsed.as_micros();
    assert!(
        avg_micros < 19_000,
        "JIT should be faster than AST interpreter (19ms). Actual: {avg_micros}µs"
    );

    // Report speedup factor
    let speedup = 19_000.0 / avg_micros as f64;
    println!("Speedup vs AST interpreter: {speedup:.1}x");

    // Stretch goal: <500µs (0.5ms) = 38x speedup
    if avg_micros < 500 {
        println!("✅ ACHIEVED stretch goal: <500µs ({avg_micros}µs)");
    } else {
        println!("⚠️  Not yet at stretch goal (<500µs), but faster than baseline");
    }
}
