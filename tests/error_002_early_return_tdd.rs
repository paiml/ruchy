// [ERROR-002] TDD tests for early return statement support
// Tests FIRST, then implementation (Toyota Way)

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Evaluate Ruchy code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

// Test 1: Simple early return in function
#[test]
fn test_simple_early_return() {
    let code = r#"
        fun safe_divide(a: i32, b: i32) -> i32 {
            if b == 0 {
                return 0;
            }
            a / b
        }

        safe_divide(10, 0)
    "#;

    let result = eval(code).expect("Should execute early return");
    assert_eq!(result.to_string(), "0");
}

// Test 2: Early return with value
#[test]
fn test_early_return_with_value() {
    let code = r#"
        fun validate_age(age: i32) -> i32 {
            if age < 0 {
                return 0;
            }
            if age > 150 {
                return 150;
            }
            age
        }

        validate_age(-5)
    "#;

    let result = eval(code).expect("Should execute early return with value");
    assert_eq!(result.to_string(), "0");
}

// Test 3: Multiple early returns
#[test]
fn test_multiple_early_returns() {
    let code = r#"
        fun classify_number(n: i32) -> &str {
            if n < 0 {
                return "negative";
            }
            if n == 0 {
                return "zero";
            }
            if n > 100 {
                return "large";
            }
            "positive"
        }

        classify_number(150)
    "#;

    let result = eval(code).expect("Should handle multiple early returns");
    assert_eq!(result.to_string(), "\"large\"");
}

// Test 4: Normal return at end still works
#[test]
fn test_normal_return_at_end() {
    let code = r#"
        fun add(a: i32, b: i32) -> i32 {
            a + b
        }

        add(5, 3)
    "#;

    let result = eval(code).expect("Should execute normal return");
    assert_eq!(result.to_string(), "8");
}

// Test 5: Early return in nested if
#[test]
fn test_early_return_nested_if() {
    let code = r#"
        fun check_range(x: i32) -> bool {
            if x < 0 {
                if x < -100 {
                    return false;
                }
                return false;
            }
            if x > 100 {
                return false;
            }
            return true;
        }

        check_range(-5)
    "#;

    let result = eval(code).expect("Should handle nested if with early return");
    assert_eq!(result.to_string(), "false");
}

// Test 6: Early return with println before it
#[test]
fn test_early_return_with_println() {
    let code = r#"
        fun safe_sqrt(x: f64) -> f64 {
            if x < 0.0 {
                println("Error: negative number");
                return 0.0;
            }
            x
        }

        safe_sqrt(-4.0)
    "#;

    let result = eval(code).expect("Should execute println then early return");
    assert_eq!(result.to_string(), "0.0");
}

// Test 7: Early return in guard clause pattern
#[test]
fn test_guard_clause_pattern() {
    let code = r#"
        fun process_data(data: &str) -> &str {
            if data.len() == 0 {
                return "empty";
            }
            if data.len() > 100 {
                return "too_long";
            }
            "valid"
        }

        process_data("")
    "#;

    let result = eval(code).expect("Should support guard clause pattern");
    assert_eq!(result.to_string(), "\"empty\"");
}

// Test 8: Early return with boolean
#[test]
fn test_early_return_boolean() {
    let code = r#"
        fun is_valid(x: i32) -> bool {
            if x <= 0 {
                return false;
            }
            if x > 1000 {
                return false;
            }
            return true;
        }

        is_valid(-1)
    "#;

    let result = eval(code).expect("Should handle early return with boolean");
    assert_eq!(result.to_string(), "false");
}

// Test 9: Early return vs final expression
#[test]
fn test_early_return_vs_final_expression() {
    let code = r#"
        fun example(flag: bool) -> i32 {
            if flag {
                return 42;
            }
            100
        }

        example(true)
    "#;

    let result = eval(code).expect("Should pick early return over final expression");
    assert_eq!(result.to_string(), "42");
}

// Test 10: Early return in while loop (should exit function, not loop)
#[test]
fn test_early_return_in_while() {
    let code = r#"
        fun find_limit() -> i32 {
            let mut i = 0;
            while i < 100 {
                if i == 5 {
                    return i;
                }
                i = i + 1;
            }
            100
        }

        find_limit()
    "#;

    let result = eval(code).expect("Should return from function, not just exit loop");
    assert_eq!(result.to_string(), "5");
}
