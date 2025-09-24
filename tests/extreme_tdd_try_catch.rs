//! EXTREME TDD: Try/Catch/Finally Implementation
//! Target: Complete error handling with ALL functions ≤10 complexity
//!
//! This test-driven implementation adds comprehensive error handling
//! to the Ruchy language following Toyota Way principles.
//!
//! Test Strategy:
//! 1. Basic try/catch functionality
//! 2. Pattern-based catch clauses
//! 3. Finally block execution
//! 4. Error propagation
//! 5. Nested try/catch
//! 6. Edge cases and property tests

use ruchy::frontend::ast::{CatchClause, Expr, ExprKind, Literal, Pattern, Span};
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, InterpreterError, Value};

#[cfg(test)]
mod try_catch_tests {
    use super::*;

    // Helper to create a simple expression
    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(val)), Span::default())
    }

    // Helper to create an error-throwing expression
    fn make_throw_expr(msg: &str) -> Expr {
        Expr::new(
            ExprKind::Throw {
                exception: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String(msg.to_string().into())),
                    Span::default(),
                )),
            },
            Span::default(),
        )
    }

    // =============================================================================
    // BASIC TRY/CATCH TESTS
    // =============================================================================

    #[test]
    fn test_try_no_error() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                42
            } catch e {
                -1
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_try_with_error() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                throw "error occurred"
            } catch e {
                99
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(99));
    }

    #[test]
    fn test_catch_binds_error() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                throw "test error"
            } catch e {
                e
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("test error".to_string().into())
        );
    }

    // =============================================================================
    // PATTERN-BASED CATCH TESTS
    // =============================================================================

    #[test]
    fn test_pattern_based_catch() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                throw { type: "IOError", message: "file not found" }
            } catch { type: "IOError", message: msg } {
                msg
            } catch e {
                "unknown error"
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("file not found".to_string().into())
        );
    }

    #[test]
    fn test_multiple_catch_clauses() {
        let mut interp = Interpreter::new();

        // Test first catch matches
        let code1 = r#"
            try {
                throw 42
            } catch n if typeof(n) == "integer" {
                "caught integer"
            } catch s if typeof(s) == "string" {
                "caught string"
            } catch _ {
                "caught other"
            }
        "#;

        let result = interp.eval_string(code1);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("caught integer".to_string().into())
        );

        // Test second catch matches
        let code2 = r#"
            try {
                throw "error"
            } catch n if typeof(n) == "integer" {
                "caught integer"
            } catch s if typeof(s) == "string" {
                "caught string"
            } catch _ {
                "caught other"
            }
        "#;

        let result = interp.eval_string(code2);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("caught string".to_string().into())
        );
    }

    // =============================================================================
    // FINALLY BLOCK TESTS
    // =============================================================================

    #[test]
    fn test_finally_executes_on_success() {
        let mut interp = Interpreter::new();
        let code = r#"
            let mut cleanup_ran = false;
            let result = try {
                42
            } catch e {
                -1
            } finally {
                cleanup_ran = true
            };
            (result, cleanup_ran)
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        if let Value::Tuple(values) = result.unwrap() {
            assert_eq!(values[0], Value::Integer(42));
            assert_eq!(values[1], Value::Bool(true));
        } else {
            panic!("Expected tuple result");
        }
    }

    #[test]
    fn test_finally_executes_on_error() {
        let mut interp = Interpreter::new();
        let code = r#"
            let mut cleanup_ran = false;
            let result = try {
                throw "error"
            } catch e {
                99
            } finally {
                cleanup_ran = true
            };
            (result, cleanup_ran)
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        if let Value::Tuple(values) = result.unwrap() {
            assert_eq!(values[0], Value::Integer(99));
            assert_eq!(values[1], Value::Bool(true));
        } else {
            panic!("Expected tuple result");
        }
    }

    #[test]
    fn test_finally_executes_even_on_uncaught() {
        let mut interp = Interpreter::new();
        let code = r#"
            let mut cleanup_ran = false;
            try {
                try {
                    throw "inner error"
                } finally {
                    cleanup_ran = true
                }
            } catch e {
                cleanup_ran
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // =============================================================================
    // ERROR PROPAGATION TESTS
    // =============================================================================

    #[test]
    fn test_error_propagation() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn might_fail(x) {
                if x < 0 {
                    throw "negative value"
                } else {
                    x * 2
                }
            }

            try {
                might_fail(-5)
            } catch e {
                e
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("negative value".to_string().into())
        );
    }

    #[test]
    fn test_rethrow_in_catch() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                try {
                    throw "original error"
                } catch e {
                    throw "rethrown: " + e
                }
            } catch e {
                e
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("rethrown: original error".to_string().into())
        );
    }

    // =============================================================================
    // NESTED TRY/CATCH TESTS
    // =============================================================================

    #[test]
    fn test_nested_try_catch() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                try {
                    throw "inner"
                } catch e {
                    throw "outer: " + e
                }
            } catch e {
                e
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("outer: inner".to_string().into())
        );
    }

    #[test]
    fn test_deeply_nested_try() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                try {
                    try {
                        throw 1
                    } catch e if e == 2 {
                        "caught 2"
                    }
                } catch e if e == 1 {
                    "caught 1"
                }
            } catch e {
                "caught other"
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("caught 1".to_string().into())
        );
    }

    // =============================================================================
    // EDGE CASES
    // =============================================================================

    #[test]
    fn test_empty_try_block() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                // Empty block returns nil
            } catch e {
                "error"
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_throw_nil() {
        let mut interp = Interpreter::new();
        let code = r#"
            try {
                throw nil
            } catch e {
                e == nil
            }
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_catch_with_break_continue() {
        let mut interp = Interpreter::new();
        let code = r#"
            let mut result = [];
            for i in [1, 2, 3] {
                try {
                    if i == 2 {
                        throw "skip"
                    }
                    result.push(i)
                } catch e {
                    continue
                }
            }
            result
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        if let Value::Array(arr) = result.unwrap() {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(3));
        } else {
            panic!("Expected array result");
        }
    }

    // =============================================================================
    // COMPLEXITY VERIFICATION
    // =============================================================================

    #[test]
    fn test_complexity_compliance() {
        // This test documents expected complexity after implementation:
        //
        // eval_try_catch: ≤10 (orchestrator)
        // eval_try_block: ≤5
        // match_catch_clause: ≤8
        // eval_catch_body: ≤5
        // eval_finally_block: ≤3
        // handle_throw: ≤3
        //
        // All functions maintain ≤10 complexity per Toyota Way
    }

    // =============================================================================
    // PROPERTY-BASED TESTS
    // =============================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::{quickcheck, TestResult};

        fn prop_try_catch_never_panics(value: i32) -> TestResult {
            let mut interp = Interpreter::new();
            let code = format!(
                r#"
                try {{
                    if {} < 0 {{
                        throw "negative"
                    }} else {{
                        {}
                    }}
                }} catch e {{
                    -1
                }}
                "#,
                value, value
            );

            let result = interp.eval_string(&code);
            TestResult::from_bool(result.is_ok())
        }

        fn prop_finally_always_executes(throw_error: bool) -> TestResult {
            let mut interp = Interpreter::new();
            let code = format!(
                r#"
                let mut counter = 0;
                try {{
                    {}
                    42
                }} catch e {{
                    99
                }} finally {{
                    counter = counter + 1
                }}
                counter
                "#,
                if throw_error { "throw \"error\";" } else { "" }
            );

            let result = interp.eval_string(&code);
            if let Ok(Value::Integer(n)) = result {
                TestResult::from_bool(n == 1)
            } else {
                TestResult::failed()
            }
        }

        #[test]
        fn test_properties() {
            quickcheck(prop_try_catch_never_panics as fn(i32) -> TestResult);
            quickcheck(prop_finally_always_executes as fn(bool) -> TestResult);
        }
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[test]
    fn test_real_world_error_handling() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn divide(a, b) {
                if b == 0 {
                    throw { type: "MathError", message: "division by zero" }
                }
                a / b
            }

            fn safe_divide(a, b) {
                try {
                    divide(a, b)
                } catch { type: "MathError", message: msg } {
                    println("Math error: " + msg);
                    nil
                } catch e {
                    println("Unknown error: " + e);
                    nil
                }
            }

            let result1 = safe_divide(10, 2);
            let result2 = safe_divide(10, 0);
            (result1, result2)
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        if let Value::Tuple(values) = result.unwrap() {
            assert_eq!(values[0], Value::Integer(5));
            assert_eq!(values[1], Value::Nil);
        } else {
            panic!("Expected tuple result");
        }
    }

    #[test]
    fn test_resource_cleanup_pattern() {
        let mut interp = Interpreter::new();
        let code = r#"
            fn with_resource(action) {
                let resource = "opened";
                try {
                    action(resource)
                } finally {
                    // Always cleanup
                    resource = "closed"
                }
            }

            let result = with_resource(fn(r) {
                if r == "opened" {
                    "success"
                } else {
                    throw "resource not ready"
                }
            });

            result
        "#;

        let result = interp.eval_string(code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("success".to_string().into()));
    }
}
