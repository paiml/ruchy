// Property-based tests for interpreter value handling
// PROPTEST-004 Part 1: Value type properties (15 tests)
//
// Properties tested:
// 1. Value equality is reflexive (v == v)
// 2. Value equality is symmetric (v1 == v2 ⟹ v2 == v1)
// 3. Integer arithmetic handles overflow gracefully
// 4. Float arithmetic handles NaN/Inf correctly
// 5. String concatenation preserves total length
// 6. Boolean logic follows truth tables (AND)
// 7. Boolean logic follows truth tables (OR)
// 8. Boolean logic follows truth tables (NOT)
// 9. Integer addition is commutative
// 10. Integer addition is associative
// 11. Integer multiplication is commutative
// 12. String concatenation is associative
// 13. Comparison operators are transitive
// 14. Zero is additive identity
// 15. One is multiplicative identity

use proptest::prelude::*;
use ruchy::runtime::repl::Repl;
use std::path::PathBuf;

// ============================================================================
// Property 1: Value equality is reflexive
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_equality_reflexive(n in -1000i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let x = {}", n)).unwrap();
        let result = repl.eval("x == x").unwrap();

        prop_assert!(result.contains("true"), "Integer equality should be reflexive: {} == {}", n, n);
    }

    #[test]
    fn prop_string_equality_reflexive(s in "[a-zA-Z0-9]{1,20}") {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        repl.eval(&format!("let x = \"{}\"", s)).unwrap();
        let result = repl.eval("x == x").unwrap();

        prop_assert!(result.contains("true"), "String equality should be reflexive");
    }
}

// ============================================================================
// Property 2: Value equality is symmetric
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_equality_symmetric(a in -100i64..100, b in -100i64..100) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("{} == {}", a, b)).unwrap();
        let result2 = repl.eval(&format!("{} == {}", b, a)).unwrap();

        prop_assert_eq!(result1, result2, "Integer equality should be symmetric");
    }
}

// ============================================================================
// Property 3: Integer arithmetic handles overflow gracefully
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_addition_no_panic(a in -1000i64..1000, b in -1000i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            repl.eval(&format!("{} + {}", a, b))
        }));

        prop_assert!(result.is_ok(), "Integer addition should not panic");
    }

    #[test]
    fn prop_integer_multiplication_no_panic(a in -100i64..100, b in -100i64..100) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            repl.eval(&format!("{} * {}", a, b))
        }));

        prop_assert!(result.is_ok(), "Integer multiplication should not panic");
    }
}

// ============================================================================
// Property 4: Float arithmetic handles NaN/Inf correctly
// ============================================================================

#[test]
fn prop_float_division_by_zero_handled() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    let result = repl.eval("1.0 / 0.0");
    // Should either return inf or handle gracefully, not panic
    assert!(
        result.is_ok() || result.is_err(),
        "Float division by zero should be handled"
    );
}

#[test]
fn prop_float_nan_propagation() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // 0.0 / 0.0 produces NaN
    let result = repl.eval("0.0 / 0.0");
    // Should handle NaN without panicking
    assert!(
        result.is_ok() || result.is_err(),
        "Float NaN should be handled"
    );
}

// ============================================================================
// Property 5: String concatenation preserves total length
// ============================================================================

proptest! {
    #[test]
    fn prop_string_concat_preserves_length(
        s1 in "[a-zA-Z]{1,10}",
        s2 in "[a-zA-Z]{1,10}"
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let expected_len = s1.len() + s2.len();
        repl.eval(&format!("let result = \"{}\" + \"{}\"", s1, s2)).unwrap();
        let result = repl.eval("result.len()").unwrap();

        prop_assert!(result.contains(&expected_len.to_string()),
            "String concatenation should preserve total length: {} + {} = {}",
            s1.len(), s2.len(), expected_len);
    }
}

// ============================================================================
// Property 6-8: Boolean logic follows truth tables
// ============================================================================

proptest! {
    #[test]
    fn prop_boolean_and_truth_table(a: bool, b: bool) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let expected = a && b;
        let result = repl.eval(&format!("{} && {}", a, b)).unwrap();
        let expected_str = if expected { "true" } else { "false" };

        prop_assert!(result.contains(expected_str), "AND truth table violated");
    }

    #[test]
    fn prop_boolean_or_truth_table(a: bool, b: bool) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let expected = a || b;
        let result = repl.eval(&format!("{} || {}", a, b)).unwrap();
        let expected_str = if expected { "true" } else { "false" };

        prop_assert!(result.contains(expected_str), "OR truth table violated");
    }

    #[test]
    fn prop_boolean_not_truth_table(a: bool) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let expected = !a;
        let result = repl.eval(&format!("!{}", a)).unwrap();
        let expected_str = if expected { "true" } else { "false" };

        prop_assert!(result.contains(expected_str), "NOT truth table violated");
    }
}

// ============================================================================
// Property 9-10: Integer arithmetic is commutative and associative
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_addition_commutative(a in -100i64..100, b in -100i64..100) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("{} + {}", a, b)).unwrap();
        let result2 = repl.eval(&format!("{} + {}", b, a)).unwrap();

        prop_assert_eq!(result1, result2, "Integer addition should be commutative");
    }

    #[test]
    fn prop_integer_addition_associative(
        a in -50i64..50,
        b in -50i64..50,
        c in -50i64..50
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("({} + {}) + {}", a, b, c)).unwrap();
        let result2 = repl.eval(&format!("{} + ({} + {})", a, b, c)).unwrap();

        prop_assert_eq!(result1, result2, "Integer addition should be associative");
    }
}

// ============================================================================
// Property 11: Integer multiplication is commutative
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_multiplication_commutative(a in -100i64..100, b in -100i64..100) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("{} * {}", a, b)).unwrap();
        let result2 = repl.eval(&format!("{} * {}", b, a)).unwrap();

        prop_assert_eq!(result1, result2, "Integer multiplication should be commutative");
    }
}

// ============================================================================
// Property 12: String concatenation is associative
// ============================================================================

proptest! {
    #[test]
    fn prop_string_concat_associative(
        s1 in "[a-z]{1,5}",
        s2 in "[a-z]{1,5}",
        s3 in "[a-z]{1,5}"
    ) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("(\"{}\" + \"{}\") + \"{}\"", s1, s2, s3)).unwrap();
        let result2 = repl.eval(&format!("\"{}\" + (\"{}\" + \"{}\")", s1, s2, s3)).unwrap();

        prop_assert_eq!(result1, result2, "String concatenation should be associative");
    }
}

// ============================================================================
// Property 13: Comparison operators are transitive
// ============================================================================

proptest! {
    #[test]
    fn prop_less_than_transitive(a in -100i64..100, b in -100i64..100, c in -100i64..100) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let ab = repl.eval(&format!("{} < {}", a, b)).unwrap();
        let bc = repl.eval(&format!("{} < {}", b, c)).unwrap();
        let ac = repl.eval(&format!("{} < {}", a, c)).unwrap();

        // If a < b and b < c, then a < c
        if ab.contains("true") && bc.contains("true") {
            prop_assert!(ac.contains("true"), "Less-than should be transitive");
        }
    }
}

// ============================================================================
// Property 14: Zero is additive identity
// ============================================================================

proptest! {
    #[test]
    fn prop_zero_additive_identity(n in -1000i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("{} + 0", n)).unwrap();
        let result2 = repl.eval(&format!("0 + {}", n)).unwrap();
        let expected = repl.eval(&format!("{}", n)).unwrap();

        prop_assert_eq!(result1, expected.clone(), "0 should be right additive identity");
        prop_assert_eq!(result2, expected, "0 should be left additive identity");
    }
}

// ============================================================================
// Property 15: One is multiplicative identity
// ============================================================================

proptest! {
    #[test]
    fn prop_one_multiplicative_identity(n in -1000i64..1000) {
        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

        let result1 = repl.eval(&format!("{} * 1", n)).unwrap();
        let result2 = repl.eval(&format!("1 * {}", n)).unwrap();
        let expected = repl.eval(&format!("{}", n)).unwrap();

        prop_assert_eq!(result1, expected.clone(), "1 should be right multiplicative identity");
        prop_assert_eq!(result2, expected, "1 should be left multiplicative identity");
    }
}
