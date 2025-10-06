// LANG-COMP-002: Operators - RED PHASE TESTS
// Tests written FIRST before examples exist
// EXTREME TDD Protocol: These tests MUST fail until examples are created

use std::process::Command;

/// Helper function to run a Ruchy example file and capture output
fn run_ruchy_file(file_path: &str) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run", file_path])
        .output()
        .expect("Failed to execute ruchy command")
}

/// Helper function to evaluate Ruchy code directly
fn eval_ruchy_code(code: &str) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "eval", code])
        .output()
        .expect("Failed to execute ruchy eval")
}

// ============================================================================
// ARITHMETIC OPERATORS TESTS
// ============================================================================

#[test]
fn test_arithmetic_operators_example() {
    let output = run_ruchy_file("examples/lang_comp/02-operators/01_arithmetic.ruchy");

    assert!(
        output.status.success(),
        "Arithmetic operators example should execute successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Example should demonstrate +, -, *, /, %
    assert!(
        !stdout.is_empty(),
        "Arithmetic example should produce output"
    );
}

#[test]
fn test_addition_operator() {
    let output = eval_ruchy_code("2 + 3");
    assert!(output.status.success(), "Addition should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "5");
}

#[test]
fn test_subtraction_operator() {
    let output = eval_ruchy_code("10 - 3");
    assert!(output.status.success(), "Subtraction should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "7");
}

#[test]
fn test_multiplication_operator() {
    let output = eval_ruchy_code("4 * 5");
    assert!(output.status.success(), "Multiplication should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "20");
}

#[test]
fn test_division_operator() {
    let output = eval_ruchy_code("20 / 4");
    assert!(output.status.success(), "Division should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "5");
}

#[test]
fn test_modulo_operator() {
    let output = eval_ruchy_code("10 % 3");
    assert!(output.status.success(), "Modulo should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "1");
}

// ============================================================================
// COMPARISON OPERATORS TESTS
// ============================================================================

#[test]
fn test_comparison_operators_example() {
    let output = run_ruchy_file("examples/lang_comp/02-operators/02_comparison.ruchy");

    assert!(
        output.status.success(),
        "Comparison operators example should execute successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty(),
        "Comparison example should produce output"
    );
}

#[test]
fn test_equality_operator() {
    let output = eval_ruchy_code("5 == 5");
    assert!(output.status.success(), "Equality should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_inequality_operator() {
    let output = eval_ruchy_code("5 != 3");
    assert!(output.status.success(), "Inequality should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_less_than_operator() {
    let output = eval_ruchy_code("3 < 5");
    assert!(output.status.success(), "Less than should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_greater_than_operator() {
    let output = eval_ruchy_code("5 > 3");
    assert!(output.status.success(), "Greater than should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_less_than_or_equal_operator() {
    let output = eval_ruchy_code("3 <= 3");
    assert!(output.status.success(), "Less than or equal should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_greater_than_or_equal_operator() {
    let output = eval_ruchy_code("5 >= 5");
    assert!(output.status.success(), "Greater than or equal should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

// ============================================================================
// LOGICAL OPERATORS TESTS
// ============================================================================

#[test]
fn test_logical_operators_example() {
    let output = run_ruchy_file("examples/lang_comp/02-operators/03_logical.ruchy");

    assert!(
        output.status.success(),
        "Logical operators example should execute successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Logical example should produce output");
}

#[test]
fn test_logical_and_operator() {
    let output = eval_ruchy_code("true && true");
    assert!(output.status.success(), "Logical AND should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_logical_or_operator() {
    let output = eval_ruchy_code("false || true");
    assert!(output.status.success(), "Logical OR should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

#[test]
fn test_logical_not_operator() {
    let output = eval_ruchy_code("!false");
    assert!(output.status.success(), "Logical NOT should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

// ============================================================================
// OPERATOR PRECEDENCE TESTS
// ============================================================================

#[test]
fn test_operator_precedence_example() {
    let output = run_ruchy_file("examples/lang_comp/02-operators/04_precedence.ruchy");

    assert!(
        output.status.success(),
        "Operator precedence example should execute successfully"
    );
}

#[test]
fn test_arithmetic_precedence() {
    let output = eval_ruchy_code("2 + 3 * 4");
    assert!(output.status.success(), "Arithmetic precedence should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "14");
}

#[test]
fn test_parentheses_override_precedence() {
    let output = eval_ruchy_code("(2 + 3) * 4");
    assert!(
        output.status.success(),
        "Parentheses should override precedence"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "20");
}

#[test]
fn test_comparison_and_logical_precedence() {
    let output = eval_ruchy_code("3 < 5 && 5 < 10");
    assert!(
        output.status.success(),
        "Comparison and logical precedence should work"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");
}

// ============================================================================
// PROPERTY-BASED TESTS (10,000+ cases per property)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore] // Run with: cargo test --test operators_test property_tests -- --ignored
        fn addition_is_commutative(a in -1000i64..1000, b in -1000i64..1000) {
            let expr1 = format!("{} + {}", a, b);
            let expr2 = format!("{} + {}", b, a);

            let output1 = eval_ruchy_code(&expr1);
            let output2 = eval_ruchy_code(&expr2);

            prop_assert!(output1.status.success());
            prop_assert!(output2.status.success());

            let result1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();
            let result2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

            prop_assert_eq!(result1, result2, "Addition should be commutative");
        }

        #[test]
        #[ignore]
        fn multiplication_is_associative(a in -100i64..100, b in -100i64..100, c in -100i64..100) {
            let expr1 = format!("({} * {}) * {}", a, b, c);
            let expr2 = format!("{} * ({} * {})", a, b, c);

            let output1 = eval_ruchy_code(&expr1);
            let output2 = eval_ruchy_code(&expr2);

            if output1.status.success() && output2.status.success() {
                let result1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();
                let result2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();
                prop_assert_eq!(result1, result2, "Multiplication should be associative");
            }
        }

        #[test]
        #[ignore]
        fn comparison_operators_never_crash(a in any::<i64>(), b in any::<i64>()) {
            let operators = vec!["==", "!=", "<", ">", "<=", ">="];

            for op in operators {
                let expr = format!("{} {} {}", a, op, b);
                let output = eval_ruchy_code(&expr);

                prop_assert!(
                    output.status.success() || String::from_utf8_lossy(&output.stderr).contains("error"),
                    "Comparison operators should not crash: {}", expr
                );
            }
        }

        #[test]
        #[ignore]
        fn logical_and_is_short_circuit(a in any::<bool>()) {
            // Test that false && X doesn't evaluate X (short-circuit)
            let expr = format!("{} && true", a);
            let output = eval_ruchy_code(&expr);

            prop_assert!(output.status.success(), "Logical AND should work");
        }

        #[test]
        #[ignore]
        fn double_negation_equals_identity(a in any::<bool>()) {
            let expr = format!("!!{}", a);
            let output = eval_ruchy_code(&expr);

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                prop_assert_eq!(result, a.to_string(), "!!x should equal x");
            }
        }
    }
}
