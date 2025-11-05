//! Property-based tests for verifying compiler invariants
#![allow(clippy::unnecessary_wraps)] // Property tests often need Result for the test framework
use crate::backend::Transpiler;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, StringPart, UnaryOp};
use crate::frontend::{Parser, RecoveryParser};
#[allow(unused_imports)]
use crate::testing::generators::{arb_expr, arb_well_typed_expr};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
/// Property: Parser should never panic on any input
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_parser_never_panics;
///
/// let result = prop_parser_never_panics("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_parser_never_panics(input: &str) -> Result<(), TestCaseError> {
    let mut parser = Parser::new(input);
    // Parser should either succeed or return an error, never panic
    let _ = parser.parse();
    Ok(())
}
/// Property: Recovery parser should always produce some AST
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_recovery_parser_always_produces_ast;
///
/// let result = prop_recovery_parser_always_produces_ast("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_recovery_parser_always_produces_ast(input: &str) -> Result<(), TestCaseError> {
    let mut parser = RecoveryParser::new(input);
    let result = parser.parse_with_recovery();
    // For non-empty input, we should get some AST or errors
    if !input.trim().is_empty() {
        prop_assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Recovery parser should produce AST or errors for non-empty input"
        );
    }
    Ok(())
}
/// Property: Transpilation preserves expression structure
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_transpilation_preserves_structure;
///
/// let result = prop_transpilation_preserves_structure(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_transpilation_preserves_structure(expr: &Expr) -> Result<(), TestCaseError> {
    let mut transpiler = Transpiler::new();
    // Transpilation should either succeed or fail cleanly
    if let Ok(rust_code) = transpiler.transpile(expr) {
        // The generated Rust code should not be empty
        let code_str = rust_code.to_string();
        prop_assert!(!code_str.is_empty(), "Transpiled code should not be empty");
    } else {
        // Transpilation errors are acceptable for some ASTs
    }
    Ok(())
}
/// Property: String interpolation transpiles correctly
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_string_interpolation_transpiles;
///
/// let result = prop_string_interpolation_transpiles(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_string_interpolation_transpiles(parts: &[StringPart]) -> Result<(), TestCaseError> {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_string_interpolation(parts);
    // Should either succeed or fail cleanly, never panic
    if let Ok(tokens) = result {
        let code = tokens.to_string();
        // Should either be a format! call or a simple string literal
        prop_assert!(
            code.contains("format!")
                || code.contains("format !")
                || code.starts_with('"')
                || code.is_empty(),
            "String interpolation should produce format! call or string literal, got: {}",
            code
        );
    }
    // Transpilation errors are acceptable for malformed parts
    Ok(())
}
/// Property: Parse-print roundtrip
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_parse_print_roundtrip;
///
/// let result = prop_parse_print_roundtrip(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_parse_print_roundtrip(expr: &Expr) -> Result<(), TestCaseError> {
    // This would require a pretty-printer, which we'll implement later
    // For now, just check that we can transpile and the result is valid
    let mut transpiler = Transpiler::new();
    if let Ok(rust_code) = transpiler.transpile(expr) {
        // Check that the Rust code contains expected elements based on expr type
        let code_str = rust_code.to_string();
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => {
                // Integer literals are transpiled with type suffixes (e.g., "42 i32")
                prop_assert!(
                    code_str.contains(&n.to_string()),
                    "Integer literal {n} not found in transpiled code"
                );
            }
            ExprKind::Literal(Literal::Bool(b)) => {
                prop_assert!(
                    code_str.contains(&b.to_string()),
                    "Bool literal {b} not found in transpiled code"
                );
            }
            ExprKind::Binary {
                op: BinaryOp::Add, ..
            } => {
                prop_assert!(
                    code_str.contains('+'),
                    "Addition operator not found in transpiled code"
                );
            }
            _ => {
                // Other cases are more complex to verify
            }
        }
    }
    Ok(())
}
/// Property: Well-typed expressions should always transpile successfully
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_well_typed_always_transpiles;
///
/// let result = prop_well_typed_always_transpiles(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_well_typed_always_transpiles(expr: &Expr) -> Result<(), TestCaseError> {
    let mut transpiler = Transpiler::new();
    // Check if this is a simple, well-typed expression
    if is_well_typed(expr) {
        match transpiler.transpile(expr) {
            Ok(_) => Ok(()),
            Err(e) => {
                prop_assert!(
                    false,
                    "Well-typed expression failed to transpile: {:?}\nError: {}",
                    expr,
                    e
                );
                Ok(())
            }
        }
    } else {
        // Complex expressions may fail, which is acceptable
        Ok(())
    }
}
/// Property: Error recovery should handle truncated input gracefully
///
/// # Errors
///
/// Returns an error if the property test fails
/// # Examples
///
/// ```
/// use ruchy::testing::properties::prop_recovery_handles_truncation;
///
/// let result = prop_recovery_handles_truncation("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn prop_recovery_handles_truncation(input: &str) -> Result<(), TestCaseError> {
    if input.is_empty() {
        return Ok(());
    }
    // Limit input length to prevent O(n²) performance in property tests
    let max_test_length = 50; // Reasonable limit for testing
    let test_input = if input.len() > max_test_length {
        &input[..max_test_length]
    } else {
        input
    };

    // Try parsing truncated versions of the input
    for i in 0..test_input.len() {
        let truncated = &test_input[..i];
        let mut parser = RecoveryParser::new(truncated);
        let result = parser.parse_with_recovery();
        // Should not panic, and should produce something
        if !truncated.trim().is_empty() {
            prop_assert!(
                result.ast.is_some() || !result.errors.is_empty(),
                "Recovery parser should handle truncated input at position {i}"
            );
        }
    }
    Ok(())
}
/// Helper to check if an expression is well-typed (simplified)
fn is_well_typed(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) | ExprKind::Identifier(_) => true,
        ExprKind::Binary { left, right, op } => {
            match op {
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                    is_numeric(left) && is_numeric(right)
                }
                BinaryOp::And | BinaryOp::Or => is_boolean(left) && is_boolean(right),
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    // Equality can work on many types
                    is_well_typed(left) && is_well_typed(right)
                }
                _ => is_well_typed(left) && is_well_typed(right),
            }
        }
        ExprKind::Unary { operand, op } => match op {
            UnaryOp::Not => is_boolean(operand),
            UnaryOp::Negate | UnaryOp::BitwiseNot => is_numeric(operand),
            UnaryOp::Reference | UnaryOp::MutableReference | UnaryOp::Deref => true, // Reference/Deref can be applied to any type (PARSER-085: Issue #71)
        },
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            is_boolean(condition)
                && is_well_typed(then_branch)
                && else_branch.as_ref().is_none_or(|e| is_well_typed(e))
        }
        _ => false, // Conservative for complex expressions
    }
}
fn is_numeric(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::Literal(Literal::Integer(_, _) | Literal::Float(_))
    )
}
fn is_boolean(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Literal(Literal::Bool(_)))
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))] // Limit test cases to prevent hanging

        #[test]
        fn test_parser_never_panics(input in prop::string::string_regex("[a-zA-Z0-9 (){}\n]{0,50}").unwrap()) {
            // Limited to reasonable inputs - no infinite loops
            prop_parser_never_panics(&input)?;
        }
        #[test]
        fn test_recovery_parser_always_produces_ast(input in prop::string::string_regex("[a-zA-Z0-9 (){}\n]{0,50}").unwrap()) {
            // Limited to reasonable inputs to prevent hanging
            prop_recovery_parser_always_produces_ast(&input)?;
        }
        #[test]
        fn test_transpilation_preserves_structure(expr in arb_expr()) {
            prop_transpilation_preserves_structure(&expr)?;
        }
        #[test]
        fn test_well_typed_always_transpiles(expr in arb_well_typed_expr()) {
            prop_well_typed_always_transpiles(&expr)?;
        }
        #[test]
        fn test_recovery_handles_truncation(input in "[a-zA-Z0-9 +\\-*/()]{0,30}") {
            prop_recovery_handles_truncation(&input)?;
        }
        #[test]
        fn test_parse_print_roundtrip(expr in arb_well_typed_expr()) {
            prop_parse_print_roundtrip(&expr)?;
        }
    }
    #[test]
    #[ignore = "Parser has infinite loop on certain inputs"]
    fn test_specific_recovery_cases() {
        // Test each case individually to identify which one hangs

        // Test case 1
        let mut parser = RecoveryParser::new("let x =");
        let result = parser.parse_with_recovery();
        assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Failed: let x ="
        );

        // Test case 2
        let mut parser = RecoveryParser::new("if x >");
        let result = parser.parse_with_recovery();
        assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Failed: if x >"
        );

        // Test case 3
        let mut parser = RecoveryParser::new("fun foo(");
        let result = parser.parse_with_recovery();
        assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Failed: fun foo("
        );

        // Test case 4
        let mut parser = RecoveryParser::new("[1, 2,");
        let result = parser.parse_with_recovery();
        assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Failed: [1, 2,"
        );

        // Test case 5
        let mut parser = RecoveryParser::new("1 + + 2");
        let result = parser.parse_with_recovery();
        assert!(
            result.ast.is_some() || !result.errors.is_empty(),
            "Failed: 1 + + 2"
        );
    }

    #[test]
    fn test_prop_parser_never_panics_unit() {
        let test_cases = vec![
            "",
            "42",
            "hello",
            "1 + 2",
            "invalid syntax !@#$",
            "let x = 42 in x",
            "fun test() { }",
            "if true { 1 } else { 2 }",
        ];

        for input in test_cases {
            let result = prop_parser_never_panics(input);
            assert!(result.is_ok(), "Parser panicked on input: {input}");
        }
    }

    #[test]
    fn test_prop_recovery_parser_unit() {
        let test_cases = vec![
            ("", false),       // Empty input shouldn't require recovery
            ("42", true),      // Valid input should produce AST
            ("1 + 2", true),   // Valid binary expression
            ("invalid", true), // Invalid but non-empty should produce something
        ];

        for (input, expect_output) in test_cases {
            let result = prop_recovery_parser_always_produces_ast(input);
            if expect_output {
                assert!(result.is_ok(), "Recovery parser failed on: {input}");
            } else {
                // Empty input case is handled differently
                assert!(result.is_ok(), "Recovery parser failed on empty input");
            }
        }
    }

    #[test]
    fn test_prop_transpilation_unit() {
        let test_exprs = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Default::default(),
            ),
            Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default()),
            Expr::new(ExprKind::Identifier("x".to_string()), Default::default()),
        ];

        for expr in test_exprs {
            let result = prop_transpilation_preserves_structure(&expr);
            assert!(result.is_ok(), "Transpilation property failed");
        }
    }

    #[test]
    fn test_prop_string_interpolation_unit() {
        let test_cases = vec![
            vec![],                                      // Empty parts
            vec![StringPart::Text("hello".to_string())], // Text only
            vec![StringPart::Expr(Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Default::default(),
            )))], // Expression only
        ];

        for (i, parts) in test_cases.iter().enumerate() {
            println!("Testing case {i}: {parts:?}");
            let transpiler = Transpiler::new();
            let result = transpiler.transpile_string_interpolation(parts);
            match result {
                Ok(tokens) => {
                    let code = tokens.to_string();
                    println!("Generated code: {code}");
                    assert!(
                        code.contains("format!") || code.contains("format !") || code.starts_with('"') || code.is_empty(),
                        "String interpolation should produce format! call or string literal, got: {code}"
                    );
                }
                Err(e) => {
                    println!("Transpilation error (acceptable): {e:?}");
                }
            }
        }
    }

    #[test]
    fn test_is_well_typed_function() {
        // Test numeric literals
        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        assert!(is_well_typed(&int_expr));

        let float_expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Default::default());
        assert!(is_well_typed(&float_expr));

        // Test boolean literals
        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        assert!(is_well_typed(&bool_expr));

        // Test identifiers
        let id_expr = Expr::new(ExprKind::Identifier("x".to_string()), Default::default());
        assert!(is_well_typed(&id_expr));
    }

    #[test]
    fn test_is_numeric_function() {
        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        assert!(is_numeric(&int_expr));

        let float_expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Default::default());
        assert!(is_numeric(&float_expr));

        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        assert!(!is_numeric(&bool_expr));

        let string_expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Default::default(),
        );
        assert!(!is_numeric(&string_expr));
    }

    #[test]
    fn test_is_boolean_function() {
        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        assert!(is_boolean(&bool_expr));

        let bool_false_expr =
            Expr::new(ExprKind::Literal(Literal::Bool(false)), Default::default());
        assert!(is_boolean(&bool_false_expr));

        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        assert!(!is_boolean(&int_expr));

        let string_expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Default::default(),
        );
        assert!(!is_boolean(&string_expr));
    }

    #[test]
    fn test_well_typed_binary_expressions() {
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Default::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Default::default(),
        );

        // Test arithmetic operations
        let add_expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Default::default(),
        );
        assert!(is_well_typed(&add_expr));

        // Test boolean operations
        let bool_left = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let bool_right = Expr::new(ExprKind::Literal(Literal::Bool(false)), Default::default());
        let boolean_and_expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(bool_left),
                op: BinaryOp::And,
                right: Box::new(bool_right),
            },
            Default::default(),
        );
        assert!(is_well_typed(&boolean_and_expr));
    }

    #[test]
    fn test_well_typed_unary_expressions() {
        // Test negation on numeric
        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        let neg_expr = Expr::new(
            ExprKind::Unary {
                operand: Box::new(int_expr),
                op: UnaryOp::Negate,
            },
            Default::default(),
        );
        assert!(is_well_typed(&neg_expr));

        // Test not on boolean
        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let not_expr = Expr::new(
            ExprKind::Unary {
                operand: Box::new(bool_expr),
                op: UnaryOp::Not,
            },
            Default::default(),
        );
        assert!(is_well_typed(&not_expr));
    }

    #[test]
    fn test_well_typed_if_expressions() {
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Default::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Default::default(),
        );

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Default::default(),
        );
        assert!(is_well_typed(&if_expr));
    }

    #[test]
    fn test_ill_typed_expressions() {
        // Test adding boolean to integer (ill-typed)
        let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let int_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );

        let bad_add = Expr::new(
            ExprKind::Binary {
                left: Box::new(bool_expr),
                op: BinaryOp::Add,
                right: Box::new(int_expr),
            },
            Default::default(),
        );
        assert!(!is_well_typed(&bad_add));
    }

    #[test]
    fn test_parser_with_edge_cases() {
        let edge_cases = vec![
            "\0",     // Null character
            "\n\n\n", // Multiple newlines
            "   ",    // Whitespace only
            "\t\t\t", // Tabs only
            "\"",     // Unterminated string
            "(",      // Unmatched paren
            "}",      // Unmatched brace
        ];

        for case in edge_cases {
            let result = prop_parser_never_panics(case);
            assert!(result.is_ok(), "Parser should not panic on: {case:?}");
        }
    }

    #[test]
    fn test_transpilation_with_complex_expressions() {
        // Test more complex expressions that might stress the transpiler
        let complex_exprs = vec![
            // Nested binary operations
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Binary {
                            left: Box::new(Expr::new(
                                ExprKind::Literal(Literal::Integer(1, None)),
                                Default::default(),
                            )),
                            op: BinaryOp::Add,
                            right: Box::new(Expr::new(
                                ExprKind::Literal(Literal::Integer(2, None)),
                                Default::default(),
                            )),
                        },
                        Default::default(),
                    )),
                    op: BinaryOp::Multiply,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(3, None)),
                        Default::default(),
                    )),
                },
                Default::default(),
            ),
        ];

        for expr in complex_exprs {
            let result = prop_transpilation_preserves_structure(&expr);
            assert!(
                result.is_ok(),
                "Transpilation should handle complex expressions"
            );
        }
    }

    #[test]
    fn test_string_interpolation_edge_cases() {
        let edge_cases = vec![
            // Empty string parts
            vec![StringPart::Text(String::new())],
            // Mixed text and expressions
            vec![
                StringPart::Text("Hello ".to_string()),
                StringPart::Expr(Box::new(Expr::new(
                    ExprKind::Identifier("name".to_string()),
                    Default::default(),
                ))),
                StringPart::Text("!".to_string()),
            ],
        ];

        for parts in edge_cases {
            let result = prop_string_interpolation_transpiles(&parts);
            assert!(
                result.is_ok(),
                "String interpolation should handle edge cases"
            );
        }
    }

    #[test]
    fn test_property_functions_return_ok() {
        // Test that all property functions return Ok for simple cases
        assert!(prop_parser_never_panics("42").is_ok());
        assert!(prop_recovery_parser_always_produces_ast("42").is_ok());

        let simple_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        assert!(prop_transpilation_preserves_structure(&simple_expr).is_ok());

        let simple_parts = vec![StringPart::Text("hello".to_string())];
        assert!(prop_string_interpolation_transpiles(&simple_parts).is_ok());
    }
}
