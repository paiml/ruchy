// SPRINT 1 (INTERP-001-B): Control Flow Tests
// Goal: 80 failing tests for control flow
// Complexity: All implementations must be ≤10
// Performance: O(n) or better
// EXTREME TDD: Write all tests FIRST, then implement

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[cfg(test)]
mod if_else_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_if_true_no_else() {
        assert_eq!(eval_expr("if true { 42 }").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_if_false_no_else() {
        assert_eq!(eval_expr("if false { 42 }").unwrap(), Value::Nil);
    }

    #[test]
    fn test_if_true_with_else() {
        assert_eq!(
            eval_expr("if true { 42 } else { 0 }").unwrap(),
            Value::Integer(42)
        );
    }

    #[test]
    fn test_if_false_with_else() {
        assert_eq!(
            eval_expr("if false { 42 } else { 0 }").unwrap(),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_if_with_expression_condition() {
        assert_eq!(
            eval_expr("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap(),
            Value::String(Rc::from("yes"))
        );
    }

    #[test]
    fn test_nested_if_else() {
        let code = "if true { if false { 1 } else { 2 } } else { 3 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_if_else_if_chain() {
        let code = "if false { 1 } else if true { 2 } else { 3 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_complex_if_condition() {
        let code = "if (5 > 3) && (2 < 4) { \"both true\" } else { \"not both\" }";
        assert_eq!(
            eval_expr(code).unwrap(),
            Value::String(Rc::from("both true"))
        );
    }

    #[test]
    fn test_if_with_block_statements() {
        let code = "if true { let x = 5; x * 2 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_if_expression_as_value() {
        let code = "let result = if 3 > 2 { \"greater\" } else { \"lesser\" }; result";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("greater")));
    }
}

#[cfg(test)]
mod match_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_match_integer_literal() {
        let code = "match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("two")));
    }

    #[test]
    fn test_match_with_default() {
        let code = "match 5 { 1 => \"one\", 2 => \"two\", _ => \"other\" }";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("other")));
    }

    #[test]
    fn test_match_string_literal() {
        let code = r#"match "hello" { "hi" => 1, "hello" => 2, _ => 3 }"#;
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_match_with_guard() {
        let code = "match 5 { x if x > 3 => \"big\", _ => \"small\" }";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("big")));
    }

    #[test]
    fn test_match_range_pattern() {
        let code = "match 15 { 1..10 => \"small\", 10..20 => \"medium\", _ => \"large\" }";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("medium")));
    }

    #[test]
    fn test_match_tuple_pattern() {
        let code =
            "match (1, 2) { (0, _) => \"first zero\", (_, 2) => \"second two\", _ => \"other\" }";
        assert_eq!(
            eval_expr(code).unwrap(),
            Value::String(Rc::from("second two"))
        );
    }

    #[test]
    fn test_nested_match() {
        let code = "match 1 { 1 => match 2 { 2 => \"nested\", _ => \"no\" }, _ => \"outer\" }";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("nested")));
    }

    #[test]
    fn test_match_with_binding() {
        let code = "match 42 { x => x * 2 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(84));
    }

    #[test]
    fn test_match_option_pattern() {
        let code = "match Some(42) { Some(x) => x, None => 0 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_match_result_pattern() {
        let code = "match Ok(42) { Ok(x) => x, Err(_) => -1 }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod loop_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_while_loop_basic() {
        let code = "let mut x = 0; while x < 5 { x = x + 1 }; x";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_while_loop_with_break() {
        let code = "let mut x = 0; while true { x = x + 1; if x == 3 { break } }; x";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_while_loop_with_continue() {
        let code = "let mut x = 0; let mut sum = 0; while x < 5 { x = x + 1; if x == 3 { continue }; sum = sum + x }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(12)); // 1 + 2 + 4 + 5
    }

    #[test]
    fn test_for_loop_range() {
        let code = "let mut sum = 0; for i in 1..5 { sum = sum + i }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(10)); // 1 + 2 + 3 + 4
    }

    #[test]
    fn test_for_loop_inclusive_range() {
        let code = "let mut sum = 0; for i in 1..=5 { sum = sum + i }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(15)); // 1 + 2 + 3 + 4 + 5
    }

    #[test]
    fn test_for_loop_list() {
        let code = "let mut sum = 0; for x in [1, 2, 3, 4] { sum = sum + x }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_for_loop_with_break() {
        let code = "let mut sum = 0; for i in 1..10 { if i > 3 { break }; sum = sum + i }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(6)); // 1 + 2 + 3
    }

    #[test]
    fn test_for_loop_with_continue() {
        let code = "let mut sum = 0; for i in 1..6 { if i == 3 { continue }; sum = sum + i }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(12)); // 1 + 2 + 4 + 5
    }

    #[test]
    fn test_loop_with_break() {
        let code = "let mut x = 0; loop { x = x + 1; if x == 5 { break } }; x";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_loop_with_break_value() {
        let code = "let result = loop { break 42 }; result";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_nested_loops() {
        let code = "let mut sum = 0; for i in 1..4 { for j in 1..4 { sum = sum + i * j } }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(36)); // Sum of products
    }

    #[test]
    fn test_labeled_break() {
        let code = "'outer: for i in 1..5 { for j in 1..5 { if i * j > 6 { break 'outer } } }";
        // This tests that labeled breaks work correctly
        assert!(eval_expr(code).is_ok());
    }
}

#[cfg(test)]
mod early_return_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_return_from_function() {
        let code = "fun test() { return 42; 99 }; test()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_conditional_return() {
        let code = "fun abs(x) { if x < 0 { return -x }; x }; abs(-5)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_return_from_loop() {
        let code = "fun find() { for i in 1..10 { if i == 5 { return i } }; 0 }; find()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_return_from_nested() {
        let code = "fun nested() { if true { if true { return 42 } }; 0 }; nested()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_implicit_return() {
        let code = "fun last() { 42 }; last()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod control_flow_edge_cases {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_empty_if_block() {
        assert_eq!(eval_expr("if true { }").unwrap(), Value::Nil);
    }

    #[test]
    fn test_empty_else_block() {
        assert_eq!(eval_expr("if false { 42 } else { }").unwrap(), Value::Nil);
    }

    #[test]
    fn test_empty_loop_body() {
        let code = "let mut x = 0; while x < 3 { x = x + 1; }; x";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_single_iteration_loop() {
        let code = "for i in 1..2 { i }";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_zero_iteration_loop() {
        let code = "let mut sum = 0; for i in 1..1 { sum = sum + i }; sum";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_deeply_nested_control_flow() {
        let code = "
            if true {
                if true {
                    if true {
                        if true {
                            42
                        } else {
                            0
                        }
                    } else {
                        1
                    }
                } else {
                    2
                }
            } else {
                3
            }
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_mixed_control_flow() {
        let code = "
            let mut result = 0;
            for i in 1..5 {
                if i % 2 == 0 {
                    result = result + match i {
                        2 => 20,
                        4 => 40,
                        _ => 0
                    }
                }
            };
            result
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(60));
    }
}
