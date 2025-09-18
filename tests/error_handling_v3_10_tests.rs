//! TDD Tests for Error Handling System
//! Sprint v3.10.0 - Result types, try operator, try/catch

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{Expr, ExprKind};
use ruchy::backend::transpiler::Transpiler;

#[cfg(test)]
mod result_type_tests {
    use super::*;

    #[test]
    fn test_parse_ok_constructor() {
        let input = "Ok(42)";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse Ok constructor");

        let ast = result.unwrap();
        if let ExprKind::Call { func, args } = &ast.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                assert_eq!(name, "Ok");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected Ok identifier");
            }
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_parse_err_constructor() {
        let input = r#"Err("error message")"#;
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse Err constructor");

        let ast = result.unwrap();
        if let ExprKind::Call { func, args } = &ast.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                assert_eq!(name, "Err");
                assert_eq!(args.len(), 1);
            } else {
                panic!("Expected Err identifier");
            }
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_function_returning_result() {
        let input = r#"
        fun divide(a: i32, b: i32) -> Result<i32, String> {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse function returning Result");
    }

    #[test]
    fn test_transpile_result_function() {
        let input = r#"
        fun safe_div(a: i32, b: i32) -> Result<i32, String> {
            if b == 0 {
                Err("Cannot divide by zero")
            } else {
                Ok(a / b)
            }
        }"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile Result function");

        let code = result.unwrap();
        assert!(code.contains("Result"));
        assert!(code.contains("Ok"));
        assert!(code.contains("Err"));
    }
}

#[cfg(test)]
mod try_operator_tests {
    use super::*;

    #[test]
    fn test_parse_try_operator() {
        let input = "value?";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse try operator");

        let ast = result.unwrap();
        if let ExprKind::Try { expr } = &ast.kind {
            if let ExprKind::Identifier(name) = &expr.kind {
                assert_eq!(name, "value");
            } else {
                panic!("Expected identifier in try expression");
            }
        } else {
            panic!("Expected Try expression, got: {:?}", ast.kind);
        }
    }

    #[test]
    fn test_try_operator_in_function() {
        let input = r#"
        fun process() -> Result<i32, String> {
            let x = get_value()?;
            Ok(x * 2)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse function with try operator");
    }

    #[test]
    fn test_chained_try_operators() {
        let input = "file.read()?.parse()?";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse chained try operators");
    }

    #[test]
    fn test_transpile_try_operator() {
        let input = "value?";
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile try operator");

        let code = result.unwrap();
        assert!(code.contains("?"), "Should contain try operator");
    }
}

#[cfg(test)]
mod try_catch_tests {
    use super::*;

    #[test]
    fn test_parse_try_catch_simple() {
        let input = r#"
        try {
            risky_operation()
        } catch (e) {
            handle_error(e)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse try/catch block");

        let ast = result.unwrap();
        if let ExprKind::TryCatch { try_block, catch_clauses, .. } = &ast.kind {
            assert_eq!(catch_clauses.len(), 1, "Should have one catch clause");
        } else {
            panic!("Expected TryCatch expression");
        }
    }

    #[test]
    fn test_try_catch_finally() {
        let input = r#"
        try {
            risky_operation()
        } catch (e) {
            handle_error(e)
        } finally {
            cleanup()
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse try/catch/finally");

        let ast = result.unwrap();
        if let ExprKind::TryCatch { finally_block, .. } = &ast.kind {
            assert!(finally_block.is_some(), "Should have finally block");
        } else {
            panic!("Expected TryCatch with finally");
        }
    }

    #[test]
    fn test_multiple_catch_clauses() {
        let input = r#"
        try {
            risky_operation()
        } catch (io_err) {
            handle_io(io_err)
        } catch (parse_err) {
            handle_parse(parse_err)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse multiple catch clauses");

        let ast = result.unwrap();
        if let ExprKind::TryCatch { catch_clauses, .. } = &ast.kind {
            assert_eq!(catch_clauses.len(), 2, "Should have two catch clauses");
        } else {
            panic!("Expected TryCatch with multiple catches");
        }
    }

    #[test]
    fn test_transpile_try_catch() {
        let input = r#"
        try {
            dangerous_op()
        } catch (e) {
            default_value()
        }"#;

        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok(), "Should transpile try/catch");

        let code = result.unwrap();
        assert!(code.contains("match"), "Should use match for error handling");
        assert!(code.contains("Ok"), "Should handle Ok case");
        assert!(code.contains("Err"), "Should handle Err case");
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    #[test]
    fn test_early_return_with_err() {
        let input = r#"
        fun validate(data: String) -> Result<i32, String> {
            if data.is_empty() {
                return Err("Data cannot be empty")
            }
            Ok(42)
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse early return with Err");
    }

    #[test]
    fn test_error_propagation_chain() {
        let input = r#"
        fun process(path: String) -> Result<String, Error> {
            let contents = read_file(path)?;
            let parsed = parse_data(contents)?;
            let validated = validate(parsed)?;
            Ok(format_output(validated))
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse error propagation chain");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_error_handling_flow() {
        let input = r#"
        fun read_config(path: String) -> Result<Config, String> {
            try {
                let contents = read_file(path)?;
                let config = parse_json(contents)?;
                Ok(config)
            } catch (e) {
                Err(format("Failed to read config: {}", e))
            }
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse complete error flow");
    }

    #[test]
    fn test_nested_try_catch() {
        let input = r#"
        try {
            try {
                inner_operation()
            } catch (inner_err) {
                recover(inner_err)
            }
        } catch (outer_err) {
            fallback()
        }"#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse nested try/catch");
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_try_operator_never_panics(expr in "[a-z][a-z0-9_]*") {
            let input = format!("{}?", expr);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_result_constructors_never_panic(value in "\"[a-zA-Z0-9 ]+\"") {
            let ok_input = format!("Ok({})", value);
            let err_input = format!("Err({})", value);

            let mut ok_parser = Parser::new(&ok_input);
            let _ = ok_parser.parse(); // Should not panic

            let mut err_parser = Parser::new(&err_input);
            let _ = err_parser.parse(); // Should not panic
        }

        #[test]
        fn test_catch_patterns_never_panic(pattern in "[a-z][a-z0-9_]*") {
            let input = format!(r#"
                try {{
                    operation()
                }} catch ({}) {{
                    handle()
                }}
            "#, pattern);

            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }
    }
}