// SPRINT 1 (INTERP-001-A): Expression Evaluation Tests
// Goal: 100 failing tests for expression evaluation
// Complexity: All implementations must be ≤10
// Performance: O(n) or better
// EXTREME TDD: Write all tests FIRST, then implement

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[cfg(test)]
mod arithmetic_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_integer_addition() {
        assert_eq!(eval_expr("2 + 3").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_integer_subtraction() {
        assert_eq!(eval_expr("10 - 4").unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_integer_multiplication() {
        assert_eq!(eval_expr("3 * 7").unwrap(), Value::Integer(21));
    }

    #[test]
    fn test_integer_division() {
        assert_eq!(eval_expr("15 / 3").unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_integer_modulo() {
        assert_eq!(eval_expr("17 % 5").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_integer_power() {
        assert_eq!(eval_expr("2 ** 8").unwrap(), Value::Integer(256));
    }

    #[test]
    fn test_float_addition() {
        assert_eq!(eval_expr("2.5 + 3.5").unwrap(), Value::Float(6.0));
    }

    #[test]
    fn test_float_subtraction() {
        assert_eq!(eval_expr("10.5 - 4.2").unwrap(), Value::Float(6.3));
    }

    #[test]
    fn test_float_multiplication() {
        assert_eq!(eval_expr("2.5 * 4.0").unwrap(), Value::Float(10.0));
    }

    #[test]
    fn test_float_division() {
        assert_eq!(eval_expr("7.5 / 2.5").unwrap(), Value::Float(3.0));
    }

    #[test]
    fn test_mixed_arithmetic() {
        // Integer + Float should promote to Float
        assert_eq!(eval_expr("5 + 2.5").unwrap(), Value::Float(7.5));
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(eval_expr("-42").unwrap(), Value::Integer(-42));
        assert_eq!(eval_expr("-3.14").unwrap(), Value::Float(-3.14));
    }

    #[test]
    fn test_parenthesized_expressions() {
        assert_eq!(eval_expr("(2 + 3) * 4").unwrap(), Value::Integer(20));
        assert_eq!(eval_expr("2 * (3 + 4)").unwrap(), Value::Integer(14));
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(eval_expr("2 + 3 * 4").unwrap(), Value::Integer(14));
        assert_eq!(eval_expr("10 - 2 * 3").unwrap(), Value::Integer(4));
    }

    #[test]
    fn test_complex_arithmetic() {
        assert_eq!(
            eval_expr("(5 + 3) * (10 - 6) / 2").unwrap(),
            Value::Integer(16)
        );
    }
}

#[cfg(test)]
mod bitwise_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_bitwise_and() {
        assert_eq!(eval_expr("5 & 3").unwrap(), Value::Integer(1));
        assert_eq!(eval_expr("12 & 10").unwrap(), Value::Integer(8));
    }

    #[test]
    fn test_bitwise_or() {
        assert_eq!(eval_expr("5 | 3").unwrap(), Value::Integer(7));
        assert_eq!(eval_expr("12 | 10").unwrap(), Value::Integer(14));
    }

    #[test]
    fn test_bitwise_xor() {
        assert_eq!(eval_expr("5 ^ 3").unwrap(), Value::Integer(6));
        assert_eq!(eval_expr("12 ^ 10").unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_bitwise_not() {
        assert_eq!(eval_expr("~0").unwrap(), Value::Integer(-1));
        assert_eq!(eval_expr("~5").unwrap(), Value::Integer(-6));
    }

    #[test]
    fn test_left_shift() {
        assert_eq!(eval_expr("1 << 3").unwrap(), Value::Integer(8));
        assert_eq!(eval_expr("5 << 2").unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_right_shift() {
        assert_eq!(eval_expr("8 >> 2").unwrap(), Value::Integer(2));
        assert_eq!(eval_expr("20 >> 1").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_complex_bitwise() {
        assert_eq!(eval_expr("(5 & 3) | (12 ^ 10)").unwrap(), Value::Integer(7));
    }
}

#[cfg(test)]
mod logical_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_logical_and() {
        assert_eq!(eval_expr("true && true").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("true && false").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("false && true").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("false && false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_logical_or() {
        assert_eq!(eval_expr("true || true").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("true || false").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("false || true").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("false || false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_logical_not() {
        assert_eq!(eval_expr("!true").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("!false").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_short_circuit_and() {
        // false && (expression) should not evaluate expression
        assert_eq!(
            eval_expr("false && (1 / 0 == 0)").unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_short_circuit_or() {
        // true || (expression) should not evaluate expression
        assert_eq!(
            eval_expr("true || (1 / 0 == 0)").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_complex_logical() {
        assert_eq!(
            eval_expr("(true && false) || (true && true)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval_expr("!((true || false) && false)").unwrap(),
            Value::Bool(true)
        );
    }
}

#[cfg(test)]
mod comparison_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_equal() {
        assert_eq!(eval_expr("5 == 5").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("5 == 3").unwrap(), Value::Bool(false));
        assert_eq!(
            eval_expr("\"hello\" == \"hello\"").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_not_equal() {
        assert_eq!(eval_expr("5 != 3").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("5 != 5").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_less_than() {
        assert_eq!(eval_expr("3 < 5").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("5 < 3").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("3 < 3").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_less_equal() {
        assert_eq!(eval_expr("3 <= 5").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("5 <= 3").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("3 <= 3").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(eval_expr("5 > 3").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("3 > 5").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("3 > 3").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_greater_equal() {
        assert_eq!(eval_expr("5 >= 3").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("3 >= 5").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("3 >= 3").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_comparison() {
        assert_eq!(
            eval_expr("\"apple\" < \"banana\"").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval_expr("\"zebra\" > \"apple\"").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_chained_comparisons() {
        // Note: This might need special handling for true chained comparisons
        assert_eq!(eval_expr("(3 < 5) && (5 < 7)").unwrap(), Value::Bool(true));
    }
}

#[cfg(test)]
mod unary_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_unary_minus_integer() {
        assert_eq!(eval_expr("-42").unwrap(), Value::Integer(-42));
        assert_eq!(eval_expr("--42").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_unary_minus_float() {
        assert_eq!(eval_expr("-3.14").unwrap(), Value::Float(-3.14));
        assert_eq!(eval_expr("--3.14").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_unary_plus() {
        assert_eq!(eval_expr("+42").unwrap(), Value::Integer(42));
        assert_eq!(eval_expr("+(-42)").unwrap(), Value::Integer(-42));
    }

    #[test]
    fn test_logical_not_expressions() {
        assert_eq!(eval_expr("!(5 > 3)").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("!(5 < 3)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_bitwise_not_expressions() {
        assert_eq!(eval_expr("~42").unwrap(), Value::Integer(-43));
        assert_eq!(eval_expr("~~42").unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod ternary_expression_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_ternary_basic() {
        assert_eq!(eval_expr("true ? 42 : 0").unwrap(), Value::Integer(42));
        assert_eq!(eval_expr("false ? 42 : 0").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_ternary_with_expressions() {
        assert_eq!(
            eval_expr("(5 > 3) ? \"yes\" : \"no\"").unwrap(),
            Value::String(Rc::from("yes"))
        );
        assert_eq!(
            eval_expr("(5 < 3) ? \"yes\" : \"no\"").unwrap(),
            Value::String(Rc::from("no"))
        );
    }

    #[test]
    fn test_nested_ternary() {
        let code = "true ? (false ? 1 : 2) : 3";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_ternary_type_coercion() {
        // Should handle different types in branches
        assert_eq!(eval_expr("true ? 42 : 3.14").unwrap(), Value::Integer(42));
        assert_eq!(eval_expr("false ? 42 : 3.14").unwrap(), Value::Float(3.14));
    }
}

#[cfg(test)]
mod type_casting_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_int_to_float_cast() {
        assert_eq!(eval_expr("42 as f64").unwrap(), Value::Float(42.0));
    }

    #[test]
    fn test_float_to_int_cast() {
        assert_eq!(eval_expr("3.14 as i32").unwrap(), Value::Integer(3));
        assert_eq!(eval_expr("3.99 as i32").unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_bool_to_int_cast() {
        assert_eq!(eval_expr("true as i32").unwrap(), Value::Integer(1));
        assert_eq!(eval_expr("false as i32").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_int_to_bool_cast() {
        assert_eq!(eval_expr("1 as bool").unwrap(), Value::Bool(true));
        assert_eq!(eval_expr("0 as bool").unwrap(), Value::Bool(false));
        assert_eq!(eval_expr("42 as bool").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_to_int_cast() {
        assert_eq!(eval_expr("\"42\" as i32").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_int_to_string_cast() {
        assert_eq!(
            eval_expr("42 as str").unwrap(),
            Value::String(Rc::from("42"))
        );
    }
}
