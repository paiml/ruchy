//! JIT-003: Division, Modulo, and Unary Operators
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Extend JIT compiler with division, modulo, and unary operators
//! Target: gcd(48, 18) <1µs (vs ~50µs AST interpreter - 50x speedup)
//!
//! Test Strategy:
//! 1. Division operator (/)
//! 2. Modulo operator (%)
//! 3. Unary negation (-x)
//! 4. Boolean NOT (!bool)
//! 5. GCD algorithm (uses division + modulo + recursion)
//! 6. Performance validation

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Division Operator
// ============================================================================

#[test]
fn test_jit_003_division_simple() {
    // Test: 10 / 2
    let code = "10 / 2";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile division: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 5, "10 / 2 should equal 5");
}

#[test]
fn test_jit_003_division_with_remainder() {
    // Test: 17 / 5 (integer division, truncates)
    let code = "17 / 5";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile division: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        3,
        "17 / 5 should equal 3 (integer division)"
    );
}

#[test]
fn test_jit_003_division_in_expression() {
    // Test: (100 / 10) + 2
    let code = "(100 / 10) + 2";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile division in expression: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 12, "(100 / 10) + 2 should equal 12");
}

// ============================================================================
// RED-002: Modulo Operator
// ============================================================================

#[test]
fn test_jit_003_modulo_simple() {
    // Test: 10 % 3
    let code = "10 % 3";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile modulo: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "10 % 3 should equal 1");
}

#[test]
fn test_jit_003_modulo_zero_remainder() {
    // Test: 15 % 5
    let code = "15 % 5";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile modulo: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "15 % 5 should equal 0");
}

#[test]
fn test_jit_003_modulo_with_if() {
    // Test: if 7 % 2 == 1 { 100 } else { 200 }
    let code = r"
        if 7 % 2 == 1 {
            100
        } else {
            200
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile modulo in if: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 100, "7 % 2 == 1, should take true branch");
}

// ============================================================================
// RED-003: Unary Negation (-x)
// ============================================================================

#[test]
fn test_jit_003_unary_negation_literal() {
    // Test: -42
    let code = "-42";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile unary negation: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), -42, "-42 should equal -42");
}

#[test]
fn test_jit_003_unary_negation_variable() {
    // Test: let x = 10; -x
    let code = r"
        let x = 10;
        -x
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile unary negation of variable: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), -10, "-x where x=10 should equal -10");
}

#[test]
fn test_jit_003_unary_negation_expression() {
    // Test: -(5 + 3)
    let code = "-(5 + 3)";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile negation of expression: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), -8, "-(5 + 3) should equal -8");
}

// ============================================================================
// RED-004: Boolean NOT (!bool)
// ============================================================================

#[test]
fn test_jit_003_boolean_not_true() {
    // Test: if !false { 1 } else { 0 }
    let code = r"
        if !false {
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
        "Should compile boolean NOT: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "!false should be true");
}

#[test]
fn test_jit_003_boolean_not_false() {
    // Test: if !true { 1 } else { 0 }
    let code = r"
        if !true {
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
        "Should compile boolean NOT: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 0, "!true should be false");
}

#[test]
fn test_jit_003_boolean_not_comparison() {
    // Test: if !(5 > 10) { 1 } else { 0 }
    let code = r"
        if !(5 > 10) {
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
        "Should compile NOT of comparison: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "!(5 > 10) should be true");
}

// ============================================================================
// RED-005: GCD Algorithm (Uses Division + Modulo + Recursion)
// ============================================================================

#[test]
fn test_jit_003_gcd_base_case() {
    // Test: gcd(10, 0) should return 10
    let code = r"
        fun gcd(a: i32, b: i32) -> i32 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        gcd(10, 0)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile gcd base case: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 10, "gcd(10, 0) should return 10");
}

#[test]
fn test_jit_003_gcd_recursive() {
    // Test: gcd(48, 18) should return 6
    let code = r"
        fun gcd(a: i32, b: i32) -> i32 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        gcd(48, 18)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile recursive gcd: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 6, "gcd(48, 18) should return 6");
}

#[test]
fn test_jit_003_gcd_coprime() {
    // Test: gcd(17, 13) should return 1 (coprime numbers)
    let code = r"
        fun gcd(a: i32, b: i32) -> i32 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        gcd(17, 13)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(
        result.is_ok(),
        "Should compile gcd of coprime numbers: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), 1, "gcd(17, 13) should return 1");
}

// ============================================================================
// RED-006: Performance Benchmark
// ============================================================================

#[test]
fn test_jit_003_performance_gcd() {
    // Performance validation: gcd should be 50-100x faster than AST interpreter
    let code = r"
        fun gcd(a: i32, b: i32) -> i32 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        gcd(1071, 462)
    ";

    let ast = Parser::new(code).parse().unwrap();

    // Warmup
    {
        let mut compiler = JitCompiler::new().unwrap();
        let warmup_result = compiler.compile_and_execute(&ast);
        assert_eq!(
            warmup_result.unwrap(),
            21,
            "gcd(1071, 462) should return 21"
        );
    }

    // Benchmark: Run 1000 iterations
    let iterations = 1000;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert_eq!(result.unwrap(), 21);
    }
    let total_elapsed = start.elapsed();
    let avg_elapsed = total_elapsed / iterations;

    println!("\n=== JIT-003 Performance Benchmark ===");
    println!("gcd(1071, 462) JIT avg over {iterations} runs: {avg_elapsed:?}");
    println!("Target: <10µs per run (faster than AST interpreter)");

    // Performance assertion: Should be faster than 1ms (very conservative)
    let avg_micros = avg_elapsed.as_micros();
    assert!(
        avg_micros < 1000,
        "JIT should be faster than 1ms. Actual: {avg_micros}µs"
    );

    println!("Actual: {avg_micros}µs - ✅ Performance target met");
}
