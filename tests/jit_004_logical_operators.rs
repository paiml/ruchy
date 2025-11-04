//! JIT-004: Logical Operators (AND, OR) with Short-Circuit Evaluation
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Extend JIT compiler with logical operators && and ||
//! Target: Short-circuit evaluation for performance and correctness
//!
//! Test Strategy:
//! 1. Logical AND (&&) - both operands true/false
//! 2. Logical OR (||) - both operands true/false
//! 3. Short-circuit AND - right side not evaluated when left is false
//! 4. Short-circuit OR - right side not evaluated when left is true
//! 5. Complex conditions - combining AND/OR with comparisons
//! 6. Nested logical expressions

#![cfg(feature = "jit")]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// RED-001: Logical AND (&&)
// ============================================================================

#[test]
fn test_jit_004_and_true_true() {
    // Test: true && true -> true (1)
    let code = r"
        if true && true {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "true && true should be true");
}

#[test]
fn test_jit_004_and_true_false() {
    // Test: true && false -> false (0)
    let code = r"
        if true && false {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "true && false should be false");
}

#[test]
fn test_jit_004_and_false_true() {
    // Test: false && true -> false (0)
    let code = r"
        if false && true {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "false && true should be false");
}

#[test]
fn test_jit_004_and_false_false() {
    // Test: false && false -> false (0)
    let code = r"
        if false && false {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "false && false should be false");
}

#[test]
fn test_jit_004_and_with_comparisons() {
    // Test: (5 > 3) && (10 < 20) -> true
    let code = r"
        if (5 > 3) && (10 < 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND with comparisons: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "(5 > 3) && (10 < 20) should be true");
}

// ============================================================================
// RED-002: Logical OR (||)
// ============================================================================

#[test]
fn test_jit_004_or_true_true() {
    // Test: true || true -> true (1)
    let code = r"
        if true || true {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "true || true should be true");
}

#[test]
fn test_jit_004_or_true_false() {
    // Test: true || false -> true (1)
    let code = r"
        if true || false {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "true || false should be true");
}

#[test]
fn test_jit_004_or_false_true() {
    // Test: false || true -> true (1)
    let code = r"
        if false || true {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "false || true should be true");
}

#[test]
fn test_jit_004_or_false_false() {
    // Test: false || false -> false (0)
    let code = r"
        if false || false {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "false || false should be false");
}

#[test]
fn test_jit_004_or_with_comparisons() {
    // Test: (5 < 3) || (10 < 20) -> true
    let code = r"
        if (5 < 3) || (10 < 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR with comparisons: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "(5 < 3) || (10 < 20) should be true");
}

// ============================================================================
// RED-003: Short-Circuit Evaluation (AND)
// ============================================================================

#[test]
fn test_jit_004_and_short_circuit_with_variables() {
    // Test: Short-circuit AND - right side uses variable set in left
    // This tests evaluation order
    let code = r"
        let x = 5;
        if (x > 10) && (x < 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile short-circuit AND: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "x=5 > 10 is false, should short-circuit");
}

#[test]
fn test_jit_004_and_both_sides_evaluated() {
    // Test: When left is true, right must be evaluated
    let code = r"
        let x = 15;
        if (x > 10) && (x < 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile AND with both sides: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "x=15: both (15 > 10) and (15 < 20) are true");
}

// ============================================================================
// RED-004: Short-Circuit Evaluation (OR)
// ============================================================================

#[test]
fn test_jit_004_or_short_circuit_with_variables() {
    // Test: Short-circuit OR - right side not needed when left is true
    let code = r"
        let x = 5;
        if (x < 10) || (x > 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile short-circuit OR: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "x=5 < 10 is true, should short-circuit to true");
}

#[test]
fn test_jit_004_or_both_sides_evaluated() {
    // Test: When left is false, right must be evaluated
    let code = r"
        let x = 15;
        if (x < 10) || (x > 20) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile OR with both sides: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "x=15: (15 < 10) is false, (15 > 20) is false");
}

// ============================================================================
// RED-005: Complex Conditions
// ============================================================================

#[test]
fn test_jit_004_complex_and_or_combination() {
    // Test: (x > 5 && x < 10) || (x == 15)
    let code = r"
        let x = 7;
        if (x > 5 && x < 10) || (x == 15) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile complex condition: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "x=7: (7 > 5 && 7 < 10) is true");
}

#[test]
fn test_jit_004_complex_condition_false() {
    // Test: (x > 5 && x < 10) || (x == 15) with x=12
    let code = r"
        let x = 12;
        if (x > 5 && x < 10) || (x == 15) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile complex condition: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "x=12: (12 > 5 && 12 < 10) is false, (12 == 15) is false");
}

#[test]
fn test_jit_004_nested_logical_operators() {
    // Test: Nested logical operators with proper precedence
    // (a || b) && (c || d)
    let code = r"
        let a = true;
        let b = false;
        let c = false;
        let d = true;
        if (a || b) && (c || d) {
            1
        } else {
            0
        }
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile nested operators: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "(true || false) && (false || true) should be true");
}

// ============================================================================
// RED-006: Range Validation (Common Use Case)
// ============================================================================

#[test]
fn test_jit_004_range_validation() {
    // Test: Check if value is in range [10, 20]
    let code = r"
        fun in_range(x: i32, min: i32, max: i32) -> i32 {
            if (x >= min) && (x <= max) {
                1
            } else {
                0
            }
        }
        in_range(15, 10, 20)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile range validation: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "15 is in range [10, 20]");
}

#[test]
fn test_jit_004_range_validation_outside() {
    // Test: Check if value is outside range
    let code = r"
        fun in_range(x: i32, min: i32, max: i32) -> i32 {
            if (x >= min) && (x <= max) {
                1
            } else {
                0
            }
        }
        in_range(5, 10, 20)
    ";
    let ast = Parser::new(code).parse().unwrap();
    let mut compiler = JitCompiler::new().unwrap();
    let result = compiler.compile_and_execute(&ast);

    assert!(result.is_ok(), "Should compile range validation: {:?}", result.err());
    assert_eq!(result.unwrap(), 0, "5 is outside range [10, 20]");
}
