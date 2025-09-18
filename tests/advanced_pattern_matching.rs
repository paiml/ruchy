//! Advanced Pattern Matching Tests
//! Sprint 64: Comprehensive test suite for advanced pattern matching features

use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

/// Test helper function to create interpreter and evaluate expression
fn eval_expr(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    interpreter.eval_string(code).map_err(|e| e.into())
}

/// Test helper to check if pattern matching works as expected
fn test_pattern_match(expr: &str, expected: &str) {
    let code = format!("match {} {{ {} }}", expr, expected);
    let result = eval_expr(&code);
    assert!(result.is_ok(), "Pattern match failed for: {}", code);
}

#[cfg(test)]
mod pattern_guard_tests {
    use super::*;

    #[test]
    fn test_simple_pattern_guard() {
        // Test: match with simple integer guard
        let code = r#"
            match 5 {
                x if x > 3 => "big",
                x => "small"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("big".to_string())));
    }

    #[test]
    fn test_pattern_guard_false_continues() {
        // Test: when guard fails, continue to next arm
        let code = r#"
            match 2 {
                x if x > 5 => "big",
                x if x > 0 => "positive",
                _ => "negative"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("positive".to_string())));
    }

    #[test]
    fn test_pattern_guard_with_destructuring() {
        // Test: guards with tuple destructuring
        let code = r#"
            match (3, 4) {
                (x, y) if x + y > 5 => "sum_big",
                (x, y) => "sum_small"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("sum_big".to_string())));
    }

    #[test]
    fn test_multiple_guards_same_pattern() {
        // Test: multiple guards on same pattern type
        let code = r#"
            match 10 {
                x if x < 5 => "small",
                x if x < 15 => "medium",
                x => "large"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("medium".to_string())));
    }

    #[test]
    fn test_guard_with_external_variable() {
        // Test: guard references external variable
        let code = r#"
            let threshold = 10;
            match 15 {
                x if x > threshold => "above",
                x => "below"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("above".to_string())));
    }
}

#[cfg(test)]
mod destructuring_tests {
    use super::*;

    #[test]
    fn test_nested_tuple_destructuring() {
        // Test: nested tuple pattern matching
        let code = r#"
            match ((1, 2), (3, 4)) {
                ((a, b), (c, d)) => a + b + c + d
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_array_destructuring_with_rest() {
        // Test: array destructuring with rest pattern
        let code = r#"
            match [1, 2, 3, 4, 5] {
                [first, second, ..rest] => first + second,
                [] => 0
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_struct_destructuring() {
        // Test: struct pattern matching
        let code = r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 3, y: 4 };
            match p {
                Point { x, y } => x * x + y * y
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(25));
    }

    #[test]
    fn test_struct_partial_destructuring() {
        // Test: struct with .. rest pattern
        let code = r#"
            struct User { name: String, age: i32, email: String }
            let user = User { name: "Alice", age: 30, email: "alice@example.com" };
            match user {
                User { name, .. } => name
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("Alice".to_string())));
    }
}

#[cfg(test)]
mod exhaustiveness_tests {
    use super::*;

    #[test]
    fn test_enum_exhaustiveness_complete() {
        // Test: complete enum coverage
        let code = r#"
            enum Color { Red, Green, Blue }
            let color = Color::Red;
            match color {
                Color::Red => "red",
                Color::Green => "green",
                Color::Blue => "blue"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("red".to_string())));
    }

    #[test]
    #[should_panic(expected = "Non-exhaustive match")]
    fn test_enum_exhaustiveness_incomplete() {
        // Test: incomplete enum coverage should fail
        let code = r#"
            enum Color { Red, Green, Blue }
            let color = Color::Blue;
            match color {
                Color::Red => "red",
                Color::Green => "green"
                // Missing Blue case - should fail exhaustiveness check
            }
        "#;
        eval_expr(code).unwrap();
    }

    #[test]
    fn test_boolean_exhaustiveness() {
        // Test: boolean exhaustiveness
        let code = r#"
            match true {
                true => "yes",
                false => "no"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("yes".to_string())));
    }

    #[test]
    fn test_integer_range_coverage() {
        // Test: integer ranges with exhaustive coverage
        let code = r#"
            match 5 {
                x if x < 0 => "negative",
                0 => "zero",
                x if x <= 10 => "small_positive",
                _ => "large_positive"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("small_positive".to_string())));
    }
}

#[cfg(test)]
mod nested_pattern_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_tuples() {
        // Test: deeply nested tuple patterns
        let code = r#"
            match (1, (2, (3, 4))) {
                (a, (b, (c, d))) => a + b + c + d
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_mixed_nested_patterns() {
        // Test: mixed nested patterns (tuples + arrays)
        let code = r#"
            match ([1, 2], (3, 4)) {
                ([a, b], (c, d)) => (a + b) * (c + d)
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(21));
    }

    #[test]
    fn test_nested_guards() {
        // Test: guards with nested destructuring
        let code = r#"
            match ((5, 3), (2, 8)) {
                ((a, b), (c, d)) if a > b && c < d => "condition_met",
                ((a, b), (c, d)) => "condition_not_met"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("condition_met".to_string())));
    }
}

#[cfg(test)]
mod or_pattern_tests {
    use super::*;

    #[test]
    fn test_simple_or_pattern() {
        // Test: simple OR pattern
        let code = r#"
            match 2 {
                1 | 2 | 3 => "low",
                4 | 5 | 6 => "medium",
                _ => "high"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("low".to_string())));
    }

    #[test]
    fn test_or_pattern_with_guards() {
        // Test: OR pattern with guards
        let code = r#"
            match 5 {
                x @ (2 | 4 | 6) if x > 3 => "even_and_big",
                x @ (1 | 3 | 5) if x > 3 => "odd_and_big",
                _ => "other"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("odd_and_big".to_string())));
    }
}

#[cfg(test)]
mod range_pattern_tests {
    use super::*;

    #[test]
    fn test_inclusive_range_pattern() {
        // Test: inclusive range patterns
        let code = r#"
            match 5 {
                1..=5 => "low_range",
                6..=10 => "high_range",
                _ => "outside"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("low_range".to_string())));
    }

    #[test]
    fn test_exclusive_range_pattern() {
        // Test: exclusive range patterns
        let code = r#"
            match 5 {
                1..5 => "low_range",
                5..10 => "high_range",
                _ => "outside"
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::String(Rc::new("high_range".to_string())));
    }
}

#[cfg(test)]
mod at_binding_tests {
    use super::*;

    #[test]
    fn test_at_binding_simple() {
        // Test: @ binding to capture matched value
        let code = r#"
            match 42 {
                x @ 42 => x,
                _ => 0
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_at_binding_with_guard() {
        // Test: @ binding with guard
        let code = r#"
            match 15 {
                x @ y if y > 10 => x * 2,
                x => x
            }
        "#;
        let result = eval_expr(code).unwrap();
        assert_eq!(result, Value::Integer(30));
    }
}

// Property-based tests using quickcheck
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn prop_wildcard_always_matches(value: i32) -> bool {
        let code = format!("match {} {{ _ => true }}", value);
        match eval_expr(&code) {
            Ok(Value::Bool(true)) => true,
            _ => false,
        }
    }

    #[quickcheck]
    fn prop_literal_pattern_matches_itself(value: i32) -> bool {
        let code = format!("match {} {{ {} => true, _ => false }}", value, value);
        match eval_expr(&code) {
            Ok(Value::Bool(true)) => true,
            _ => false,
        }
    }

    #[quickcheck]
    fn prop_guard_evaluation_consistent(value: i32) -> quickcheck::TestResult {
        if value.abs() > 1000 { return quickcheck::TestResult::discard(); }

        let code = format!("match {} {{ x if x > 0 => true, _ => false }}", value);
        let result = eval_expr(&code).unwrap();
        let expected = Value::Bool(value > 0);

        quickcheck::TestResult::from_bool(result == expected)
    }
}