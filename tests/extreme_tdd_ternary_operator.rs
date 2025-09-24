// EXTREME TDD: Ternary Operator Implementation
// STOP THE LINE - Missing feature detected
// Goal: Implement ternary operator (condition ? true_expr : false_expr)
// Complexity: ≤10 per function
// Zero SATD, Zero entropy

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::ExprKind;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[cfg(test)]
mod ternary_parser_tests {
    use super::*;

    #[test]
    fn test_parse_simple_ternary() {
        let mut parser = Parser::new("true ? 1 : 0");
        let result = parser.parse();
        assert!(result.is_ok());

        let expr = result.unwrap();
        // Should parse as Ternary/Conditional expression
        match &expr.kind {
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                // Verify structure
                assert!(matches!(condition.kind, ExprKind::Literal(_)));
                assert!(matches!(true_expr.kind, ExprKind::Literal(_)));
                assert!(matches!(false_expr.kind, ExprKind::Literal(_)));
            }
            _ => panic!("Expected ternary expression, got {:?}", expr.kind),
        }
    }

    #[test]
    fn test_parse_nested_ternary() {
        let mut parser = Parser::new("a > b ? (c > d ? 1 : 2) : 3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ternary_with_complex_condition() {
        let mut parser = Parser::new("(x > 5 && y < 10) ? \"yes\" : \"no\"");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ternary_precedence() {
        // Ternary has low precedence, so this should parse as (2 + 3) ? 4 : 5
        let mut parser = Parser::new("2 + 3 ? 4 : 5");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ternary_right_associative() {
        // Should parse as a ? b : (c ? d : e)
        let mut parser = Parser::new("a ? b : c ? d : e");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod ternary_interpreter_tests {
    use super::*;

    fn eval(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_eval_ternary_true() {
        assert_eq!(eval("true ? 42 : 0").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_ternary_false() {
        assert_eq!(eval("false ? 42 : 0").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_eval_ternary_with_expression_condition() {
        assert_eq!(
            eval("5 > 3 ? \"yes\" : \"no\"").unwrap(),
            Value::String(Rc::from("yes"))
        );
        assert_eq!(
            eval("5 < 3 ? \"yes\" : \"no\"").unwrap(),
            Value::String(Rc::from("no"))
        );
    }

    #[test]
    fn test_eval_nested_ternary() {
        assert_eq!(
            eval("true ? (false ? 1 : 2) : 3").unwrap(),
            Value::Integer(2)
        );
        assert_eq!(
            eval("false ? 1 : (true ? 2 : 3)").unwrap(),
            Value::Integer(2)
        );
    }

    #[test]
    fn test_eval_ternary_different_types() {
        // Can return different types
        assert_eq!(eval("true ? 42 : 3.14").unwrap(), Value::Integer(42));
        assert_eq!(eval("false ? 42 : 3.14").unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_eval_ternary_short_circuit() {
        // Should not evaluate the branch not taken
        assert_eq!(eval("true ? 42 : (1/0)").unwrap(), Value::Integer(42));
        assert_eq!(eval("false ? (1/0) : 42").unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod ternary_transpiler_tests {
    use super::*;

    fn transpile(code: &str) -> Result<String, String> {
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        let transpiler = Transpiler::new();
        transpiler
            .transpile_expr(&expr)
            .map(|tokens| tokens.to_string())
            .map_err(|e| e.to_string())
    }

    #[test]
    fn test_transpile_simple_ternary() {
        let result = transpile("true ? 1 : 0").unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("true"));
        assert!(result.contains('1'));
        assert!(result.contains("else"));
        assert!(result.contains('0'));
    }

    #[test]
    fn test_transpile_ternary_expression() {
        let result = transpile("x > 5 ? \"big\" : \"small\"").unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("x > 5"));
        assert!(result.contains("\"big\""));
        assert!(result.contains("\"small\""));
    }

    #[test]
    fn test_transpile_nested_ternary() {
        let result = transpile("a ? (b ? 1 : 2) : 3").unwrap();
        // Should generate nested if expressions
        assert!(result.contains("if"));
    }

    #[test]
    fn test_transpile_ternary_in_assignment() {
        let code = "let result = condition ? true_val : false_val";
        let mut parser = Parser::new(code);
        let expr = parser.parse();
        assert!(expr.is_ok());

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&expr.unwrap());
        assert!(result.is_ok());
    }
}
