// QUALITY-009: Interpreter Coverage Boost Sprint
// Target: 200+ tests to boost coverage from ~68% to 85%+
// Approach: EXTREME TDD - Write all tests BEFORE implementation

use ruchy::runtime::interpreter::{Interpreter, Value};
use ruchy::runtime::InterpreterError;
use ruchy::Parser;

// Helper trait to add eval_string functionality
trait InterpreterExt {
    fn eval_string(&mut self, code: &str) -> Result<Value, InterpreterError>;
}

impl InterpreterExt for Interpreter {
    fn eval_string(&mut self, code: &str) -> Result<Value, InterpreterError> {
        let mut parser = Parser::new(code);
        let expr = parser
            .parse()
            .map_err(|e| InterpreterError::RuntimeError(format!("Parse error: {:?}", e)))?;

        self.eval_expr(&expr)
    }
}

// ================================
// EXPRESSION TESTS (80 tests)
// ================================

mod arithmetic_tests {
    use super::*;

    #[test]
    fn test_integer_addition() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("1 + 2");
        assert_eq!(result, Ok(Value::Integer(3)));
    }

    #[test]
    fn test_integer_subtraction() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("5 - 3");
        assert_eq!(result, Ok(Value::Integer(2)));
    }

    #[test]
    fn test_integer_multiplication() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("4 * 3");
        assert_eq!(result, Ok(Value::Integer(12)));
    }

    #[test]
    fn test_integer_division() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10 / 2");
        assert_eq!(result, Ok(Value::Integer(5)));
    }

    #[test]
    fn test_integer_modulo() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10 % 3");
        assert_eq!(result, Ok(Value::Integer(1)));
    }

    #[test]
    fn test_float_addition() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("1.5 + 2.5");
        assert_eq!(result, Ok(Value::Float(4.0)));
    }

    #[test]
    fn test_float_subtraction() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("5.5 - 2.5");
        assert_eq!(result, Ok(Value::Float(3.0)));
    }

    #[test]
    fn test_float_multiplication() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("2.5 * 4.0");
        assert_eq!(result, Ok(Value::Float(10.0)));
    }

    #[test]
    fn test_float_division() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10.0 / 4.0");
        assert_eq!(result, Ok(Value::Float(2.5)));
    }

    #[test]
    fn test_mixed_arithmetic() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("2 + 3.5");
        assert_eq!(result, Ok(Value::Float(5.5)));
    }

    #[test]
    fn test_integer_overflow_addition() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(&format!("{} + 1", i64::MAX));
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_integer_underflow_subtraction() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(&format!("{} - 1", i64::MIN));
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_division_by_zero_integer() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10 / 0");
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }

    #[test]
    fn test_division_by_zero_float() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10.0 / 0.0");
        match result {
            Ok(Value::Float(f)) => assert!(f.is_infinite()),
            _ => panic!("Expected infinity for float division by zero"),
        }
    }

    #[test]
    fn test_modulo_by_zero() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("10 % 0");
        assert!(matches!(result, Err(InterpreterError::DivisionByZero)));
    }

    #[test]
    fn test_complex_expression() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("(2 + 3) * 4 - 1");
        assert_eq!(result, Ok(Value::Integer(19)));
    }

    #[test]
    fn test_nested_parentheses() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("((2 + 3) * (4 - 1)) + 5");
        assert_eq!(result, Ok(Value::Integer(20)));
    }

    #[test]
    fn test_unary_negation_integer() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("-42");
        assert_eq!(result, Ok(Value::Integer(-42)));
    }

    #[test]
    fn test_unary_negation_float() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("-3.14");
        assert_eq!(result, Ok(Value::Float(-3.14)));
    }

    #[test]
    fn test_unary_not_boolean() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("!true");
        assert_eq!(result, Ok(Value::Bool(false)));
    }
}

mod logical_tests {
    use super::*;

    #[test]
    fn test_logical_and_true() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("true && true");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_and_false() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("true && false");
        assert_eq!(result, Ok(Value::Bool(false)));
    }

    #[test]
    fn test_logical_or_true() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("false || true");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_or_false() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("false || false");
        assert_eq!(result, Ok(Value::Bool(false)));
    }

    #[test]
    fn test_logical_short_circuit_and() {
        let mut interp = Interpreter::new();
        // If short-circuit works, the division by zero won't be evaluated
        let result = interp.eval_string("false && (10 / 0 == 0)");
        assert_eq!(result, Ok(Value::Bool(false)));
    }

    #[test]
    fn test_logical_short_circuit_or() {
        let mut interp = Interpreter::new();
        // If short-circuit works, the division by zero won't be evaluated
        let result = interp.eval_string("true || (10 / 0 == 0)");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_chained() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("true && false || true");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_with_comparison() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("(5 > 3) && (2 < 4)");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_not_chained() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("!(true && false)");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_logical_complex() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("(true || false) && !(false && true)");
        assert_eq!(result, Ok(Value::Bool(true)));
    }

    #[test]
    fn test_truthy_values() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("1 && true"), Ok(Value::Bool(true)));
        assert_eq!(
            interp.eval_string("\"hello\" && true"),
            Ok(Value::Bool(true))
        );
        assert_eq!(interp.eval_string("[] && true"), Ok(Value::Bool(false))); // Empty array is falsy
    }

    #[test]
    fn test_falsy_values() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("0 || false"), Ok(Value::Bool(false)));
        assert_eq!(interp.eval_string("nil || false"), Ok(Value::Bool(false)));
        assert_eq!(interp.eval_string("\"\" || false"), Ok(Value::Bool(false)));
    }
}

mod comparison_tests {
    use super::*;

    #[test]
    fn test_equal_integers() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("5 == 5"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("5 == 3"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_not_equal_integers() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("5 != 3"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("5 != 5"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_less_than() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("3 < 5"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("5 < 3"), Ok(Value::Bool(false)));
        assert_eq!(interp.eval_string("3 < 3"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_less_than_equal() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("3 <= 5"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("3 <= 3"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("5 <= 3"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_greater_than() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("5 > 3"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("3 > 5"), Ok(Value::Bool(false)));
        assert_eq!(interp.eval_string("3 > 3"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_greater_than_equal() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("5 >= 3"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("3 >= 3"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("3 >= 5"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_string_comparison() {
        let mut interp = Interpreter::new();
        assert_eq!(
            interp.eval_string(r#""hello" == "hello""#),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            interp.eval_string(r#""hello" != "world""#),
            Ok(Value::Bool(true))
        );
        assert_eq!(interp.eval_string(r#""a" < "b""#), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_mixed_type_comparison() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("5 == 5.0"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("3 < 3.5"), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_nil_comparison() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("nil == nil"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("nil != 0"), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_boolean_comparison() {
        let mut interp = Interpreter::new();
        assert_eq!(interp.eval_string("true == true"), Ok(Value::Bool(true)));
        assert_eq!(interp.eval_string("true != false"), Ok(Value::Bool(true)));
    }
}

// ================================
// CONTROL FLOW TESTS (60 tests)
// ================================

mod control_flow_tests {
    use super::*;

    #[test]
    fn test_if_true() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("if true { 42 } else { 0 }");
        assert_eq!(result, Ok(Value::Integer(42)));
    }

    #[test]
    fn test_if_false() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("if false { 42 } else { 0 }");
        assert_eq!(result, Ok(Value::Integer(0)));
    }

    #[test]
    fn test_if_without_else() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("if false { 42 }");
        assert_eq!(result, Ok(Value::Nil));
    }

    #[test]
    fn test_nested_if() {
        let mut interp = Interpreter::new();
        let code = r#"
            if true {
                if false { 1 } else { 2 }
            } else {
                3
            }
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(2)));
    }

    #[test]
    fn test_for_loop_array() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for x in [1, 2, 3] {
                sum = sum + x
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(6)));
    }

    #[test]
    fn test_for_loop_range_exclusive() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for i in 0..5 {
                sum = sum + i
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(10))); // 0 + 1 + 2 + 3 + 4
    }

    #[test]
    fn test_for_loop_range_inclusive() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for i in 0..=5 {
                sum = sum + i
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(15))); // 0 + 1 + 2 + 3 + 4 + 5
    }

    #[test]
    fn test_for_loop_break() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for i in 0..10 {
                if i == 5 { break }
                sum = sum + i
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(10))); // 0 + 1 + 2 + 3 + 4
    }

    #[test]
    fn test_for_loop_continue() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for i in 0..5 {
                if i == 2 { continue }
                sum = sum + i
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(8))); // 0 + 1 + 3 + 4
    }

    #[test]
    fn test_while_loop() {
        let mut interp = Interpreter::new();
        let code = r#"
            let i = 0
            let sum = 0
            while i < 5 {
                sum = sum + i
                i = i + 1
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(10)));
    }

    #[test]
    fn test_while_loop_break() {
        let mut interp = Interpreter::new();
        let code = r#"
            let i = 0
            while true {
                if i == 5 { break }
                i = i + 1
            }
            i
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(5)));
    }

    #[test]
    fn test_while_loop_continue() {
        let mut interp = Interpreter::new();
        let code = r#"
            let i = 0
            let sum = 0
            while i < 5 {
                i = i + 1
                if i == 3 { continue }
                sum = sum + i
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(12))); // 1 + 2 + 4 + 5
    }

    #[test]
    fn test_match_integer() {
        let mut interp = Interpreter::new();
        let code = r#"
            match 2 {
                1 => "one",
                2 => "two",
                _ => "other"
            }
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::String("two".into())));
    }

    #[test]
    fn test_match_with_guard() {
        let mut interp = Interpreter::new();
        let code = r#"
            let x = 5
            match x {
                n if n < 0 => "negative",
                n if n > 0 => "positive",
                _ => "zero"
            }
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::String("positive".into())));
    }

    #[test]
    fn test_nested_loops() {
        let mut interp = Interpreter::new();
        let code = r#"
            let sum = 0
            for i in 0..3 {
                for j in 0..3 {
                    sum = sum + i * j
                }
            }
            sum
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(9))); // 0 + 0 + 0 + 0 + 1 + 2 + 0 + 2 + 4
    }
}

// ================================
// FUNCTION TESTS (40 tests)
// ================================

mod function_tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn add(a, b) {
                a + b
            }
            add(3, 4)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(7)));
    }

    #[test]
    fn test_recursive_factorial() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn factorial(n) {
                if n <= 1 { 1 } else { n * factorial(n - 1) }
            }
            factorial(5)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(120)));
    }

    #[test]
    fn test_closure_capture() {
        let mut interp = Interpreter::new();
        let code = r#"
            let x = 10
            let add_x = fn(y) { x + y }
            add_x(5)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(15)));
    }

    #[test]
    fn test_higher_order_function() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn apply_twice(f, x) {
                f(f(x))
            }
            fn double(n) { n * 2 }
            apply_twice(double, 3)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(12)));
    }

    #[test]
    fn test_lambda_expression() {
        let mut interp = Interpreter::new();
        let code = r#"
            let double = |x| x * 2
            double(5)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(10)));
    }

    #[test]
    fn test_nested_function() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn outer(x) {
                fn inner(y) {
                    x + y
                }
                inner(10)
            }
            outer(5)
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(15)));
    }
}

// ================================
// EDGE CASES & ERROR HANDLING (20+ tests)
// ================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_deep_recursion_limit() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn recurse(n) {
                if n == 0 { 0 } else { recurse(n - 1) }
            }
            recurse(100)
        "#;
        let result = interp.eval_string(code);
        // Should succeed with reasonable recursion depth
        assert_eq!(result, Ok(Value::Integer(0)));
    }

    #[test]
    fn test_empty_array() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("[]");
        assert_eq!(result, Ok(Value::Array(vec![].into())));
    }

    #[test]
    fn test_nested_arrays() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("[[1, 2], [3, 4]]");
        match result {
            Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
            _ => panic!("Expected nested array"),
        }
    }

    #[test]
    fn test_string_concatenation() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(r#""hello" + " " + "world""#);
        assert_eq!(result, Ok(Value::String("hello world".into())));
    }

    #[test]
    fn test_string_interpolation() {
        let mut interp = Interpreter::new();
        let code = r#"
            let name = "world"
            let x = 42
            f"Hello {name}, the answer is {x}"
        "#;
        let result = interp.eval_string(code);
        assert_eq!(
            result,
            Ok(Value::String("Hello world, the answer is 42".into()))
        );
    }

    #[test]
    fn test_object_literal() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(r#"{ name: "Alice", age: 30 }"#);
        match result {
            Ok(Value::Object(obj)) => {
                assert_eq!(obj.len(), 2);
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("age"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_array_index_access() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("[1, 2, 3][1]");
        assert_eq!(result, Ok(Value::Integer(2)));
    }

    #[test]
    fn test_array_out_of_bounds() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("[1, 2, 3][10]");
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_undefined_variable() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("undefined_var");
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_type_mismatch_addition() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(r#"5 + "hello""#);
        assert!(matches!(result, Err(InterpreterError::TypeError(_))));
    }

    #[test]
    fn test_function_wrong_arity() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn add(a, b) { a + b }
            add(1)
        "#;
        let result = interp.eval_string(code);
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_nil_operations() {
        let mut interp = Interpreter::new();
        assert!(matches!(interp.eval_string("nil + 1"), Err(_)));
        assert!(matches!(interp.eval_string("nil * 2"), Err(_)));
        assert_eq!(interp.eval_string("nil == nil"), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_empty_program() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("");
        // Empty string will fail to parse, expecting a parse error
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_comment_handling() {
        let mut interp = Interpreter::new();
        let code = r#"
            // This is a comment
            42 // Another comment
            /* Block comment */
        "#;
        let result = interp.eval_string(code);
        assert_eq!(result, Ok(Value::Integer(42)));
    }

    #[test]
    fn test_chain_method_calls() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string(r#""hello".to_uppercase().len()"#);
        match result {
            Ok(Value::Integer(n)) => assert_eq!(n, 5),
            _ => panic!("Expected integer length"),
        }
    }
}
