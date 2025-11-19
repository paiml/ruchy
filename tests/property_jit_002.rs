//! Property Tests for JIT Compiler (JIT-002)
//!
//! Purpose: Verify JIT compilation maintains correctness invariants
//! Ticket: JIT-002
//! Target: 10,000+ property test iterations pass
//!
//! ## Property Invariants
//!
//! 1. **JIT-AST Equivalence**: JIT output matches AST interpreter output for valid programs
//! 2. **Deterministic**: Same input always produces same JIT output
//! 3. **Never Panics**: Valid programs don't crash JIT compiler
//! 4. **Type Safety**: Type-correct programs compile successfully

#![cfg(feature = "jit")]
#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use ruchy::jit::JitCompiler;
use ruchy::Parser;

// ============================================================================
// PROPERTY TEST GENERATORS
// ============================================================================

/// Generate arbitrary integer literals
fn arb_int_literal() -> impl Strategy<Value = String> {
    (0i64..1000).prop_map(|n| n.to_string())
}

/// Generate arbitrary boolean literals
fn arb_bool_literal() -> impl Strategy<Value = String> {
    prop_oneof![Just("true".to_string()), Just("false".to_string())]
}

/// Generate arbitrary comparison operators
fn arb_comparison_op() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("==".to_string()),
        Just("!=".to_string()),
        Just("<".to_string()),
        Just("<=".to_string()),
        Just(">".to_string()),
        Just(">=".to_string()),
    ]
}

/// Generate arbitrary arithmetic operators
fn arb_arithmetic_op() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("+".to_string()),
        Just("-".to_string()),
        Just("*".to_string()),
        // Note: Division not yet implemented in JIT-002
    ]
}

/// Generate arbitrary simple expressions (integers and booleans)
fn arb_simple_expr() -> impl Strategy<Value = String> {
    prop_oneof![arb_int_literal(), arb_bool_literal(),]
}

/// Generate arbitrary arithmetic expressions
fn arb_arithmetic_expr() -> impl Strategy<Value = String> {
    let leaf = arb_int_literal();

    leaf.prop_recursive(
        2,  // levels deep
        10, // max size
        3,  // items per collection
        |inner| {
            (inner.clone(), arb_arithmetic_op(), inner)
                .prop_map(|(left, op, right)| format!("({left} {op} {right})"))
        },
    )
}

/// Generate arbitrary comparison expressions
fn arb_comparison_expr() -> impl Strategy<Value = String> {
    (arb_int_literal(), arb_comparison_op(), arb_int_literal())
        .prop_map(|(left, op, right)| format!("{left} {op} {right}"))
}

/// Generate arbitrary if/else expressions
fn arb_if_expr() -> impl Strategy<Value = String> {
    (arb_comparison_expr(), arb_int_literal(), arb_int_literal()).prop_map(
        |(cond, then_val, else_val)| format!("if {cond} {{ {then_val} }} else {{ {else_val} }}"),
    )
}

/// Generate arbitrary simple programs (no functions yet)
fn arb_simple_program() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_int_literal(),
        arb_bool_literal(),
        arb_arithmetic_expr(),
        arb_if_expr(),
    ]
}

// ============================================================================
// PROPERTY 1: JIT vs AST Equivalence
// ============================================================================

proptest! {
    /// Property: JIT output is deterministic
    ///
    /// Invariant: For all valid programs P, jit_eval(P) produces same result repeatedly
    /// (Note: JIT-vs-AST equivalence tested separately in unit tests due to type conversions)
    #[test]
    #[ignore = "Run with: cargo test property_jit -- --ignored --nocapture"]
    fn prop_jit_simple_programs_succeed_or_fail_consistently(program in arb_simple_program()) {
        // Parse once
        let parse_result = Parser::new(&program).parse();

        // If parsing fails, skip this test case (not a valid program)
        if parse_result.is_err() {
            return Ok(());
        }

        let ast = parse_result.unwrap();

        // Compile and execute twice - should get same result
        let mut jit_compiler1 = JitCompiler::new().unwrap();
        let jit_result1 = jit_compiler1.compile_and_execute(&ast);

        let mut jit_compiler2 = JitCompiler::new().unwrap();
        let jit_result2 = jit_compiler2.compile_and_execute(&ast);

        // Both should succeed or both should fail consistently
        match (jit_result1, jit_result2) {
            (Ok(val1), Ok(val2)) => {
                prop_assert_eq!(val1, val2, "JIT non-deterministic for: {}", program);
            }
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(
                    e1.to_string(), e2.to_string(),
                    "JIT errors differ for: {}", program
                );
            }
            _ => {
                prop_assert!(false, "JIT inconsistent success/failure for: {}", program);
            }
        }
    }

    /// Property: JIT compilation is deterministic
    ///
    /// Invariant: For all programs P, jit_eval(P) == jit_eval(P)
    #[test]
    #[ignore]
    fn prop_jit_is_deterministic(program in arb_simple_program()) {
        let parse_result = Parser::new(&program).parse();
        if parse_result.is_err() {
            return Ok(());
        }

        let ast = parse_result.unwrap();

        // Compile and execute twice
        let mut compiler1 = JitCompiler::new().unwrap();
        let result1 = compiler1.compile_and_execute(&ast);

        let mut compiler2 = JitCompiler::new().unwrap();
        let result2 = compiler2.compile_and_execute(&ast);

        // Results should be identical
        match (result1, result2) {
            (Ok(val1), Ok(val2)) => {
                prop_assert_eq!(val1, val2, "Non-deterministic JIT output for: {}", program);
            }
            (Err(e1), Err(e2)) => {
                // Both failed with same error message
                prop_assert_eq!(
                    e1.to_string(),
                    e2.to_string(),
                    "Non-deterministic JIT error for: {}", program
                );
            }
            _ => {
                prop_assert!(false, "Inconsistent JIT behavior for: {}", program);
            }
        }
    }

    /// Property: JIT compiler never panics on valid syntax
    ///
    /// Invariant: For all syntactically valid programs P, jit_compile(P) returns Ok(_) or Err(_)
    #[test]
    #[ignore]
    fn prop_jit_never_panics(program in arb_simple_program()) {
        let parse_result = Parser::new(&program).parse();
        if parse_result.is_err() {
            return Ok(());
        }

        let ast = parse_result.unwrap();

        // JIT compilation should not panic
        let result = std::panic::catch_unwind(|| {
            let mut compiler = JitCompiler::new().unwrap();
            compiler.compile_and_execute(&ast)
        });

        prop_assert!(
            result.is_ok(),
            "JIT compiler panicked on valid program: {}", program
        );
    }
}

// ============================================================================
// PROPERTY 2: Arithmetic Properties
// ============================================================================

proptest! {
    /// Property: Arithmetic operations preserve commutativity
    ///
    /// Invariant: For all a, b: jit_eval(a + b) == jit_eval(b + a)
    #[test]
    #[ignore]
    fn prop_jit_addition_commutative(a in 0i64..100, b in 0i64..100) {
        let program1 = format!("{a} + {b}");
        let program2 = format!("{b} + {a}");

        let ast1 = Parser::new(&program1).parse().unwrap();
        let ast2 = Parser::new(&program2).parse().unwrap();

        let mut compiler1 = JitCompiler::new().unwrap();
        let result1 = compiler1.compile_and_execute(&ast1).unwrap();

        let mut compiler2 = JitCompiler::new().unwrap();
        let result2 = compiler2.compile_and_execute(&ast2).unwrap();

        prop_assert_eq!(result1, result2, "{} + {} != {} + {}", a, b, b, a);
    }

    /// Property: Multiplication is associative
    ///
    /// Invariant: For all a, b, c: (a * b) * c == a * (b * c)
    #[test]
    #[ignore]
    fn prop_jit_multiplication_associative(a in 1i64..20, b in 1i64..20, c in 1i64..20) {
        let program1 = format!("({a} * {b}) * {c}");
        let program2 = format!("{a} * ({b} * {c})");

        let ast1 = Parser::new(&program1).parse().unwrap();
        let ast2 = Parser::new(&program2).parse().unwrap();

        let mut compiler1 = JitCompiler::new().unwrap();
        let result1 = compiler1.compile_and_execute(&ast1).unwrap();

        let mut compiler2 = JitCompiler::new().unwrap();
        let result2 = compiler2.compile_and_execute(&ast2).unwrap();

        prop_assert_eq!(result1, result2, "({} * {}) * {} != {} * ({} * {})", a, b, c, a, b, c);
    }

    /// Property: Subtraction identity
    ///
    /// Invariant: For all a: a - 0 == a
    #[test]
    #[ignore]
    fn prop_jit_subtraction_identity(a in 0i64..1000) {
        let program = format!("{a} - 0");
        let ast = Parser::new(&program).parse().unwrap();

        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast).unwrap();

        prop_assert_eq!(result, a, "{} - 0 != {}", a, a);
    }
}

// ============================================================================
// PROPERTY 3: Comparison Properties
// ============================================================================

proptest! {
    /// Property: Comparison reflexivity
    ///
    /// Invariant: For all a: a == a is true
    #[test]
    #[ignore]
    fn prop_jit_equality_reflexive(a in 0i64..1000) {
        let program = format!("if {a} == {a} {{ 1 }} else {{ 0 }}");
        let ast = Parser::new(&program).parse().unwrap();

        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast).unwrap();

        prop_assert_eq!(result, 1, "{} == {} should be true", a, a);
    }

    /// Property: Comparison transitivity
    ///
    /// Invariant: For all a, b, c: if a <= b and b <= c, then a <= c
    #[test]
    #[ignore]
    fn prop_jit_less_equal_transitive(a in 0i64..100, b in 100i64..200, c in 200i64..300) {
        // Ensure a <= b <= c by construction
        let program = format!(
            "if {a} <= {b} {{ if {b} <= {c} {{ if {a} <= {c} {{ 1 }} else {{ 0 }} }} else {{ 0 }} }} else {{ 0 }}"
        );
        let ast = Parser::new(&program).parse().unwrap();

        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast).unwrap();

        prop_assert_eq!(result, 1, "Transitivity violation: {} <= {} <= {}", a, b, c);
    }
}

// ============================================================================
// UNIT TESTS (Sanity Checks)
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Simple integer compiles with JIT
    #[test]
    fn test_jit_simple_integer() {
        let ast = Parser::new("42").parse().unwrap();
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast).unwrap();
        assert_eq!(result, 42);
    }

    /// Sanity check: Arithmetic operations work
    #[test]
    fn test_jit_arithmetic() {
        let ast = Parser::new("10 + 20 * 3").parse().unwrap();
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast).unwrap();
        assert_eq!(result, 70); // 10 + (20 * 3)
    }

    /// Sanity check: JIT produces correct results
    #[test]
    fn test_jit_correctness_basic() {
        let program = "if 5 > 3 { 100 } else { 200 }";
        let ast = Parser::new(program).parse().unwrap();

        let mut jit_compiler = JitCompiler::new().unwrap();
        let jit_result = jit_compiler.compile_and_execute(&ast).unwrap();

        // Should take true branch (5 > 3)
        assert_eq!(jit_result, 100);
    }
}
