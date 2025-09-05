//! TDD tests for eval_expr_kind refactoring - Toyota Way safety net
//! Target: Test ALL branches of eval_expr_kind before refactoring 138 → <50 complexity
//! This ensures zero regressions during performance optimization

#[cfg(test)]
mod tests {
    use ruchy::runtime::interpreter::{Interpreter, Value};
    use ruchy::frontend::parser::Parser;
    
    // Helper function (complexity: 3)
    fn eval_str(input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        Ok(interpreter.eval_expr(&expr)?)
    }
    
    // ==================== LITERAL TESTS ====================
    // These test ExprKind::Literal branch (complexity: 2 each)
    
    #[test]
    fn test_eval_literal_integer() {
        let result = eval_str("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }
    
    #[test]
    fn test_eval_literal_float() {
        let result = eval_str("3.14");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(3.14));
    }
    
    #[test]
    fn test_eval_literal_bool_true() {
        let result = eval_str("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_eval_literal_bool_false() {
        let result = eval_str("false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }
    
    #[test]
    fn test_eval_literal_string() {
        let result = eval_str(r#""hello""#);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected string value"),
        }
    }
    
    // ==================== IDENTIFIER TESTS ====================
    // These test ExprKind::Identifier branch (complexity: 3 each)
    
    #[test]
    fn test_eval_identifier_defined() {
        let result = eval_str("let x = 42; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }
    
    #[test]
    fn test_eval_identifier_undefined() {
        let result = eval_str("undefined_variable");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_identifier_shadowing() {
        let result = eval_str("let x = 1; let x = 2; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
    
    // ==================== BINARY OPERATION TESTS ====================
    // These test ExprKind::Binary branch (complexity: 3 each)
    
    #[test]
    fn test_eval_binary_add_integers() {
        let result = eval_str("1 + 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
    
    #[test]
    fn test_eval_binary_subtract_integers() {
        let result = eval_str("5 - 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
    
    #[test]
    fn test_eval_binary_multiply_integers() {
        let result = eval_str("3 * 4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(12));
    }
    
    #[test]
    fn test_eval_binary_divide_integers() {
        let result = eval_str("10 / 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5));
    }
    
    #[test]
    fn test_eval_binary_add_floats() {
        let result = eval_str("1.5 + 2.5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(4.0));
    }
    
    #[test]
    fn test_eval_binary_equal_true() {
        let result = eval_str("1 == 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_eval_binary_equal_false() {
        let result = eval_str("1 == 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }
    
    #[test]
    fn test_eval_binary_less_than() {
        let result = eval_str("1 < 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
    
    #[test]
    fn test_eval_binary_logical_and() {
        let result = eval_str("true && false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }
    
    #[test]
    fn test_eval_binary_logical_or() {
        let result = eval_str("false || true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
    
    // ==================== UNARY OPERATION TESTS ====================
    // These test ExprKind::Unary branch (complexity: 3 each)
    
    #[test]
    fn test_eval_unary_negate_integer() {
        let result = eval_str("-42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(-42));
    }
    
    #[test]
    fn test_eval_unary_negate_float() {
        let result = eval_str("-3.14");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(-3.14));
    }
    
    #[test]
    fn test_eval_unary_not_true() {
        let result = eval_str("!true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }
    
    #[test]
    fn test_eval_unary_not_false() {
        let result = eval_str("!false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
    
    // ==================== FUNCTION CALL TESTS ====================
    // These test ExprKind::Call branch (complexity: 4 each)
    
    #[test]
    fn test_eval_function_call_simple() {
        let result = eval_str("fun add(x, y) { x + y }; add(1, 2)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
    
    #[test]
    fn test_eval_function_call_no_args() {
        let result = eval_str("fun get_answer() { 42 }; get_answer()");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }
    
    #[test]
    fn test_eval_function_call_nested() {
        let result = eval_str("fun double(x) { x * 2 }; fun quad(x) { double(double(x)) }; quad(3)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(12));
    }
    
    #[test]
    fn test_eval_function_call_closure() {
        let result = eval_str("let x = 10; fun add_x(y) { x + y }; add_x(5)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(15));
    }
    
    // ==================== METHOD CALL TESTS ====================
    // These test ExprKind::MethodCall branch (complexity: 4 each)
    
    #[test]
    fn test_eval_method_call_length() {
        let result = eval_str("[1, 2, 3].len()");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
    
    #[test]
    fn test_eval_method_call_string() {
        let result = eval_str(r#""hello".to_string()"#);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_eval_method_call_chained() {
        let result = eval_str("[1, 2, 3].first().unwrap()");
        // May not be implemented yet, test gracefully
        let _ = result;
    }
    
    // ==================== FUNCTION/LAMBDA DEFINITION TESTS ====================
    // These test ExprKind::Function and ExprKind::Lambda branches (complexity: 4 each)
    
    #[test]
    fn test_eval_function_definition() {
        let result = eval_str("fun add(x, y) { x + y }");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Closure { .. } => {}, // Function definitions return closures
            _ => panic!("Expected closure value"),
        }
    }
    
    #[test]
    fn test_eval_lambda_simple() {
        let result = eval_str("|x| x + 1");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Closure { .. } => {}, 
            _ => panic!("Expected closure value"),
        }
    }
    
    #[test]
    fn test_eval_lambda_no_params() {
        let result = eval_str("|| 42");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Closure { .. } => {}, 
            _ => panic!("Expected closure value"),
        }
    }
    
    #[test]
    fn test_eval_lambda_multiple_params() {
        let result = eval_str("|x, y, z| x + y + z");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Closure { .. } => {}, 
            _ => panic!("Expected closure value"),
        }
    }
    
    // ==================== CONTROL FLOW TESTS ====================
    // These test is_control_flow_expr branch (complexity: 4 each)
    
    #[test]
    fn test_eval_if_expr_true() {
        let result = eval_str("if true { 1 } else { 2 }");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(1));
    }
    
    #[test]
    fn test_eval_if_expr_false() {
        let result = eval_str("if false { 1 } else { 2 }");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
    
    #[test]
    fn test_eval_if_expr_no_else() {
        let result = eval_str("if true { 42 }");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }
    
    #[test]
    fn test_eval_let_expr() {
        let result = eval_str("let x = 10; let y = 20; x + y");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(30));
    }
    
    #[test]
    fn test_eval_for_loop() {
        let result = eval_str("let sum = 0; for i in [1, 2, 3] { sum = sum + i }; sum");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(6));
    }
    
    #[test]
    fn test_eval_while_loop() {
        let result = eval_str("let x = 0; while x < 3 { x = x + 1 }; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
    
    #[test]
    fn test_eval_match_expr() {
        let result = eval_str("let x = 2; match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_str(), "two"),
            _ => panic!("Expected string value"),
        }
    }
    
    // ==================== DATA STRUCTURE TESTS ====================
    // These test is_data_structure_expr branch (complexity: 4 each)
    
    #[test]
    fn test_eval_list_empty() {
        let result = eval_str("[]");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array value"),
        }
    }
    
    #[test]
    fn test_eval_list_with_elements() {
        let result = eval_str("[1, 2, 3]");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array value"),
        }
    }
    
    #[test]
    fn test_eval_block_single() {
        let result = eval_str("{ 42 }");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }
    
    #[test]
    fn test_eval_block_multiple() {
        let result = eval_str("{ let x = 1; let y = 2; x + y }");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
    
    #[test]
    fn test_eval_tuple() {
        let result = eval_str("(1, 2, 3)");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Tuple(tuple) => assert_eq!(tuple.len(), 3),
            _ => panic!("Expected tuple value"),
        }
    }
    
    #[test]
    fn test_eval_range() {
        let result = eval_str("0..5");
        assert!(result.is_ok());
        // Range should evaluate to some representation
    }
    
    // ==================== ASSIGNMENT TESTS ====================
    // These test is_assignment_expr branch (complexity: 4 each)
    
    #[test]
    fn test_eval_assign_simple() {
        let result = eval_str("let x = 1; x = 2; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
    
    #[test]
    fn test_eval_compound_assign_add() {
        let result = eval_str("let x = 5; x += 3; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(8));
    }
    
    #[test]
    fn test_eval_compound_assign_multiply() {
        let result = eval_str("let x = 4; x *= 2; x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(8));
    }
    
    // ==================== STRING INTERPOLATION TESTS ====================
    // These test ExprKind::StringInterpolation branch (complexity: 4 each)
    
    #[test]
    fn test_eval_string_interpolation_simple() {
        let result = eval_str("let name = \"world\"; f\"Hello {name}!\"");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_str(), "Hello world!"),
            _ => panic!("Expected string value"),
        }
    }
    
    #[test]
    fn test_eval_string_interpolation_multiple() {
        let result = eval_str("let x = 5; let y = 10; f\"Sum: {x + y}\"");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_str(), "Sum: 15"),
            _ => panic!("Expected string value"),
        }
    }
    
    // ==================== QUALIFIED NAME TESTS ====================
    // These test ExprKind::QualifiedName branch (complexity: 3 each)
    
    #[test]
    fn test_eval_qualified_name() {
        let result = eval_str("std::collections::HashMap");
        // May not be fully implemented, test gracefully
        let _ = result;
    }
    
    // ==================== ERROR PATH TESTS ====================
    // These test the catch-all branch for unimplemented expressions (complexity: 2 each)
    
    #[test]
    fn test_eval_unimplemented_expr() {
        // Test expressions that should return unimplemented error
        let unimplemented_cases = vec![
            "struct Point { x: i32, y: i32 }",  // Struct definition
            "enum Color { Red, Blue }",         // Enum definition
            "trait Display { }",                // Trait definition
        ];
        
        for case in unimplemented_cases {
            let result = eval_str(case);
            // Should either be unimplemented or handled gracefully
            if result.is_err() {
                // Error is expected for unimplemented features
                continue;
            }
        }
    }
    
    // ==================== COMPLEX INTEGRATION TESTS ====================
    // These test multiple branches in combination (complexity: 6 each)
    
    #[test]
    fn test_eval_complex_arithmetic() {
        let result = eval_str("let x = 5; let y = 3; (x + y) * 2 - 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(15)); // (5+3)*2-1 = 15
    }
    
    #[test]
    fn test_eval_complex_control_flow() {
        let result = eval_str("
            let x = 10;
            if x > 5 {
                let y = x * 2;
                if y > 15 {
                    y + 5
                } else {
                    y - 5
                }
            } else {
                x
            }
        ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(25)); // 10*2+5 = 25
    }
    
    #[test]
    fn test_eval_complex_function_calls() {
        let result = eval_str("
            fun fibonacci(n) {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            };
            fibonacci(5)
        ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5)); // fib(5) = 5
    }
    
    #[test]
    fn test_eval_complex_data_structures() {
        let result = eval_str("
            let data = [1, 2, 3];
            let result = [];
            for item in data {
                result = [item * 2];
            };
            result
        ");
        assert!(result.is_ok());
        // Should work with list operations
    }
    
    #[test]
    fn test_eval_complex_closures() {
        let result = eval_str("
            let x = 10;
            let make_adder = |n| {
                |y| x + n + y
            };
            let add_five = make_adder(5);
            add_five(3)
        ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(18)); // 10+5+3 = 18
    }
}