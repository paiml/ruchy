//! Interpreter inline tests - extracted from interpreter.rs for coverage improvement

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::bool_assert_comparison)]
#[allow(clippy::approx_constant)]
#[allow(clippy::panic)]
mod tests {
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::interpreter::*;
    use crate::runtime::Value;

    #[test]
    fn test_value_creation() {
        let int_val = Value::from_i64(42);
        assert_eq!(int_val.as_i64().expect("Should be integer"), 42);
        assert_eq!(int_val.type_name(), "integer");

        let bool_val = Value::from_bool(true);
        assert_eq!(bool_val.as_bool().expect("Should be boolean"), true);
        assert_eq!(bool_val.type_name(), "boolean");

        let nil_val = Value::nil();
        assert!(nil_val.is_nil());
        assert_eq!(nil_val.type_name(), "nil");

        let float_val = Value::from_f64(3.15);
        let f_value = float_val.as_f64().expect("Should be float");
        assert!((f_value - 3.15).abs() < f64::EPSILON);
        assert_eq!(float_val.type_name(), "float");

        let string_val = Value::from_string("hello".to_string());
        assert_eq!(string_val.type_name(), "string");
    }

    #[test]
    fn test_arithmetic() {
        let mut interp = Interpreter::new();

        // Test 2 + 3 = 5
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_i64(3)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());

        let result = interp.pop().expect("Stack should not be empty");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_mixed_arithmetic() {
        let mut interp = Interpreter::new();

        // Test 2 + 3.5 = 5.5 (int + float -> float)
        assert!(interp.push(Value::from_i64(2)).is_ok());
        assert!(interp.push(Value::from_f64(3.5)).is_ok());
        assert!(interp.binary_op(BinaryOp::Add).is_ok());

        let result = interp.pop().expect("Stack should not be empty");
        match result {
            Value::Float(f) => assert!((f - 5.5).abs() < f64::EPSILON),
            _ => unreachable!("Expected float, got {result:?}"),
        }
    }
}

#[cfg(test)]
mod lambda_tests {
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::Value;

    #[test]
    fn test_lambda_variable_assignment_and_call() {
        let code = r"
            let double = x => x * 2
            double(5)
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_lambda_pipe_syntax_variable_call() {
        let code = r"
            let triple = |x| x * 3
            triple(4)
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(12));
    }
}

#[cfg(test)]
mod negative_indexing_tests {
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::Value;

    // FEATURE-042 (GitHub Issue #46): Negative indexing tests

    #[test]
    fn test_negative_array_indexing_last_element() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-1]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("cherry".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_second_to_last() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-2]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("banana".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_first_element() {
        let code = r#"
            let fruits = ["apple", "banana", "cherry"]
            fruits[-3]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("apple".to_string()));
    }

    #[test]
    fn test_negative_array_indexing_out_of_bounds() {
        let code = r#"
            let fruits = ["apple", "banana"]
            fruits[-5]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);
        assert!(
            result.is_err(),
            "Should fail for out-of-bounds negative index"
        );
    }

    #[test]
    fn test_negative_string_indexing() {
        let code = r#"
            let word = "hello"
            word[-1]
        "#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::from_string("o".to_string()));
    }

    #[test]
    fn test_negative_tuple_indexing() {
        let code = r"
            let point = (10, 20, 30)
            point[-1]
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_negative_indexing_with_integers() {
        let code = r"
            let numbers = [100, 200, 300, 400]
            numbers[-2]
        ";
        let mut parser = crate::frontend::parser::Parser::new(code);
        let ast = parser.parse().expect("Parse failed");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Eval failed");
        assert_eq!(result, Value::Integer(300));
    }
}

// Tests removed - moved to separate test files

/// Coverage boost tests for uncovered interpreter paths
/// EXTREME TDD Round 85: Tests for type definitions, special forms, imports

#[cfg(test)]
mod coverage_boost_tests {
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::Value;

    include!("interpreter_inline_tests_part1.rs");
    include!("interpreter_inline_tests_part2.rs");
}
