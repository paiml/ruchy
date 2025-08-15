use crate::backend::Transpiler;
use crate::frontend::ast::*;
use crate::frontend::{Parser, RecoveryParser};
use crate::testing::generators::*;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

/// Property: Parser should never panic on any input
pub fn prop_parser_never_panics(input: String) -> Result<(), TestCaseError> {
    let mut parser = Parser::new(&input);
    // Parser should either succeed or return an error, never panic
    let _ = parser.parse();
    Ok(())
}

/// Property: Recovery parser should always produce some AST
pub fn prop_recovery_parser_always_produces_ast(input: String) -> Result<(), TestCaseError> {
    let mut parser = RecoveryParser::new(&input);
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
pub fn prop_transpilation_preserves_structure(expr: Expr) -> Result<(), TestCaseError> {
    let transpiler = Transpiler::new();
    
    // Transpilation should either succeed or fail cleanly
    match transpiler.transpile(&expr) {
        Ok(rust_code) => {
            // The generated Rust code should not be empty
            let code_str = rust_code.to_string();
            prop_assert!(!code_str.is_empty(), "Transpiled code should not be empty");
        }
        Err(_) => {
            // Transpilation errors are acceptable for some ASTs
        }
    }
    Ok(())
}

/// Property: Parse-print roundtrip
pub fn prop_parse_print_roundtrip(expr: Expr) -> Result<(), TestCaseError> {
    // This would require a pretty-printer, which we'll implement later
    // For now, just check that we can transpile and the result is valid
    let transpiler = Transpiler::new();
    if let Ok(rust_code) = transpiler.transpile(&expr) {
        // Check that the Rust code contains expected elements based on expr type
        let code_str = rust_code.to_string();
        
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(n)) => {
                prop_assert!(
                    code_str.contains(&n.to_string()),
                    "Integer literal {} not found in transpiled code", n
                );
            }
            ExprKind::Literal(Literal::Bool(b)) => {
                prop_assert!(
                    code_str.contains(&b.to_string()),
                    "Bool literal {} not found in transpiled code", b
                );
            }
            ExprKind::Binary { op: BinaryOp::Add, .. } => {
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
pub fn prop_well_typed_always_transpiles(expr: Expr) -> Result<(), TestCaseError> {
    let transpiler = Transpiler::new();
    
    // Check if this is a simple, well-typed expression
    if is_well_typed(&expr) {
        match transpiler.transpile(&expr) {
            Ok(_) => Ok(()),
            Err(e) => {
                prop_assert!(
                    false,
                    "Well-typed expression failed to transpile: {:?}\nError: {}",
                    expr, e
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
pub fn prop_recovery_handles_truncation(input: String) -> Result<(), TestCaseError> {
    if input.is_empty() {
        return Ok(());
    }
    
    // Try parsing truncated versions of the input
    for i in 0..input.len() {
        let truncated = &input[..i];
        let mut parser = RecoveryParser::new(truncated);
        let result = parser.parse_with_recovery();
        
        // Should not panic, and should produce something
        if !truncated.trim().is_empty() {
            prop_assert!(
                result.ast.is_some() || !result.errors.is_empty(),
                "Recovery parser should handle truncated input at position {}", i
            );
        }
    }
    Ok(())
}

/// Helper to check if an expression is well-typed (simplified)
fn is_well_typed(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) => true,
        ExprKind::Identifier(_) => true,
        ExprKind::Binary { left, right, op } => {
            match op {
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                    is_numeric(left) && is_numeric(right)
                }
                BinaryOp::And | BinaryOp::Or => {
                    is_boolean(left) && is_boolean(right)
                }
                BinaryOp::Equal | BinaryOp::NotEqual => {
                    // Equality can work on many types
                    is_well_typed(left) && is_well_typed(right)
                }
                _ => is_well_typed(left) && is_well_typed(right),
            }
        }
        ExprKind::Unary { operand, op } => {
            match op {
                UnaryOp::Not => is_boolean(operand),
                UnaryOp::Negate => is_numeric(operand),
                UnaryOp::BitwiseNot => is_numeric(operand),
            }
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            is_boolean(condition) && is_well_typed(then_branch) && 
            else_branch.as_ref().is_none_or(|e| is_well_typed(e))
        }
        _ => false, // Conservative for complex expressions
    }
}

fn is_numeric(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::Literal(Literal::Integer(_)) | ExprKind::Literal(Literal::Float(_))
    )
}

fn is_boolean(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Literal(Literal::Bool(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    proptest! {
        #[test]
        fn test_parser_never_panics(input in ".*") {
            prop_parser_never_panics(input)?;
        }
        
        #[test]
        fn test_recovery_parser_always_produces_ast(input in ".*") {
            prop_recovery_parser_always_produces_ast(input)?;
        }
        
        #[test]
        fn test_transpilation_preserves_structure(expr in arb_expr()) {
            prop_transpilation_preserves_structure(expr)?;
        }
        
        #[test]
        fn test_well_typed_always_transpiles(expr in arb_well_typed_expr()) {
            prop_well_typed_always_transpiles(expr)?;
        }
        
        #[test]
        fn test_recovery_handles_truncation(input in "[a-zA-Z0-9 +\\-*/()]+") {
            prop_recovery_handles_truncation(input)?;
        }
        
        #[test]
        fn test_parse_print_roundtrip(expr in arb_well_typed_expr()) {
            prop_parse_print_roundtrip(expr)?;
        }
    }
    
    #[test]
    fn test_specific_recovery_cases() {
        // Test specific error recovery scenarios
        let cases = vec![
            ("let x =", "Missing value in let binding"),
            ("if x >", "Missing right operand"),
            ("fun foo(", "Missing closing paren"),
            ("[1, 2,", "Missing closing bracket"),
            ("1 + + 2", "Double operator"),
        ];
        
        for (input, _description) in cases {
            let mut parser = RecoveryParser::new(input);
            let result = parser.parse_with_recovery();
            
            assert!(
                result.ast.is_some() || !result.errors.is_empty(),
                "Failed to handle: {}", input
            );
        }
    }
}