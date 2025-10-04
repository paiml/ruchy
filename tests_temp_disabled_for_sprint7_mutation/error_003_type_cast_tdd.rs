// [ERROR-003] TDD tests for type casting (as operator)
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

// Test 1: Cast integer to float
#[test]
fn test_cast_i32_to_f64() {
    let code = r"
        let x: i32 = 42;
        x as f64
    ";

    let result = eval(code).expect("Should cast i32 to f64");
    assert_eq!(result.to_string(), "42.0");
}

// Test 2: Cast in arithmetic expression
#[test]
fn test_cast_in_arithmetic() {
    let code = r"
        let total = 10;
        let count = 3;
        (total as f64) / (count as f64)
    ";

    let result = eval(code).expect("Should cast in division");
    // 10 / 3 = 3.333...
    assert!(result.to_string().starts_with("3.3"));
}

// Test 3: Cast in function parameter
#[test]
fn test_cast_in_function_call() {
    let code = r"
        fun calculate(principal: f64, months: i32) -> f64 {
            principal / (months as f64)
        }

        calculate(100000.0, 360)
    ";

    let result = eval(code).expect("Should cast in function");
    // 100000 / 360 = 277.777...
    assert!(result.to_string().starts_with("277."));
}

// Test 4: Cast float to integer (truncation)
#[test]
fn test_cast_f64_to_i32() {
    let code = r"
        let x = 42.7;
        x as i32
    ";

    let result = eval(code).expect("Should cast f64 to i32");
    assert_eq!(result.to_string(), "42");
}

// Test 5: Cast in complex expression
#[test]
fn test_cast_complex_expression() {
    let code = r"
        let rate = 0.05;
        let months = 360;
        let monthly_rate = rate / 12.0;

        monthly_rate * ((1.0 + monthly_rate).powf(months as f64))
    ";

    let result = eval(code).expect("Should handle cast in powf");
    // Result should be a float
    assert!(result.to_string().contains('.'));
}

// Test 6: Multiple casts in same expression
#[test]
fn test_multiple_casts() {
    let code = r"
        let a: i32 = 10;
        let b: i32 = 3;
        (a as f64) / (b as f64)
    ";

    let result = eval(code).expect("Should handle multiple casts");
    assert!(result.to_string().starts_with("3.3"));
}

// Test 7: Cast with variables
#[test]
fn test_cast_variable() {
    let code = r"
        let months = 12;
        let year_fraction = months as f64 / 12.0;
        year_fraction
    ";

    let result = eval(code).expect("Should cast variable");
    assert_eq!(result.to_string(), "1.0");
}

// Test 8: Cast in return statement
#[test]
fn test_cast_in_return() {
    let code = r"
        fun get_ratio(total: i32, count: i32) -> f64 {
            if count == 0 {
                return 0.0;
            }
            (total as f64) / (count as f64)
        }

        get_ratio(100, 3)
    ";

    let result = eval(code).expect("Should cast in return expression");
    assert!(result.to_string().starts_with("33."));
}
