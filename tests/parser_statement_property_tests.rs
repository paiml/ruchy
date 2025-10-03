// Property-based tests for parser statement handling
// PROPTEST-003 Part 2: Statement parsing properties (10 tests)
//
// Properties tested:
// 1. Variable declarations parse correctly
// 2. Function definitions preserve signatures
// 3. Block statements nest properly
// 4. Control flow structures parse correctly
// 5. Return statements preserve expressions
// 6. Assignment statements parse correctly
// 7. Expression statements wrap correctly
// 8. Import/export statements parse correctly
// 9. Struct definitions preserve fields
// 10. Match expressions parse arms correctly

use proptest::prelude::*;
use ruchy::frontend::ast::{ExprKind, Literal};
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property 1: Variable declarations parse correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_let_declarations_parse_with_value(
        value in 1i64..1000
    ) {
        let code = format!("let x = {value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse let declaration: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Let { .. } = expr.kind {
                // Correct - let declaration parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected let expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_mut_declarations_parse(
        value in 1i64..1000
    ) {
        let code = format!("let mut x = {value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse let mut declaration: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Let { is_mutable, .. } = expr.kind {
                prop_assert!(is_mutable, "Expected mutable variable");
            } else {
                return Err(TestCaseError::fail(format!("Expected let expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 2: Function definitions preserve signatures
// ============================================================================

proptest! {
    #[test]
    fn prop_function_definitions_parse(
        param_name in "[a-z][a-z0-9]{0,5}",
        return_val in 1i64..100
    ) {
        // Skip reserved keywords
        if is_reserved_keyword(&param_name) {
            return Ok(());
        }

        let code = format!("fn test({param_name}) {{ {return_val} }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse function definition: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Function { params, .. } = &expr.kind {
                prop_assert_eq!(params.len(), 1, "Function should have 1 parameter");
            } else {
                return Err(TestCaseError::fail(format!("Expected function definition, got {:?}", expr.kind)));
            }
        }
    }
}

#[test]
fn prop_zero_param_functions_parse() {
    let code = "fn test() { 42 }";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse zero-param function");
    if let Ok(expr) = result {
        if let ExprKind::Function { params, .. } = &expr.kind {
            assert_eq!(params.len(), 0, "Function should have 0 parameters");
        } else {
            panic!("Expected function definition, got {:?}", expr.kind);
        }
    }
}

// ============================================================================
// Property 3: Block statements nest properly
// ============================================================================

proptest! {
    #[test]
    fn prop_nested_blocks_parse(depth in 1usize..5) {
        // Generate { { { 42 } } }
        let mut code = "42".to_string();
        for _ in 0..depth {
            code = format!("{{ {code} }}");
        }

        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse nested blocks with depth {}: {}", depth, code);
        // Blocks should successfully parse and preserve inner value
    }
}

// ============================================================================
// Property 4: Control flow structures parse correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_if_expressions_parse(
        condition in 1i64..100,
        then_val in 1i64..100,
        else_val in 1i64..100
    ) {
        let code = format!("if {condition} {{ {then_val} }} else {{ {else_val} }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse if expression: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::If { .. } = expr.kind {
                // Correct - if expression parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected if expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_while_loops_parse(
        condition in 1i64..100,
        body_val in 1i64..100
    ) {
        let code = format!("while {condition} {{ {body_val} }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse while loop: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::While { .. } = expr.kind {
                // Correct - while loop parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected while expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_for_loops_parse(
        start in 1i64..50,
        end in 51i64..100
    ) {
        let code = format!("for i in {start}..{end} {{ i }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse for loop: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::For { .. } = expr.kind {
                // Correct - for loop parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected for expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 5: Return statements preserve expressions
// ============================================================================

proptest! {
    #[test]
    fn prop_return_statements_preserve_values(value in 1i64..1000) {
        let code = format!("return {value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse return expression: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Return { value: Some(ret_expr) } = &expr.kind {
                if let ExprKind::Literal(Literal::Integer(n)) = ret_expr.kind {
                    prop_assert_eq!(n, value, "Return value mismatch");
                } else {
                    return Err(TestCaseError::fail(format!("Expected integer literal in return, got {:?}", ret_expr.kind)));
                }
            } else {
                return Err(TestCaseError::fail(format!("Expected return expression, got {:?}", expr.kind)));
            }
        }
    }
}

#[test]
fn prop_return_without_value_parses() {
    let code = "return";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse empty return expression");
    if let Ok(expr) = result {
        if let ExprKind::Return { value: None } = expr.kind {
            // Correct - empty return parsed
        } else {
            panic!(
                "Expected return expression without value, got {:?}",
                expr.kind
            );
        }
    }
}

// ============================================================================
// Property 6: Assignment statements parse correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_simple_assignments_parse(
        value in 1i64..1000
    ) {
        let code = format!("x = {value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse assignment: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Assign { .. } = expr.kind {
                // Correct - assignment parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected assignment expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_field_assignments_parse(
        value in 1i64..1000
    ) {
        let code = format!("obj.field = {value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse field assignment: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Assign { .. } = expr.kind {
                // Correct - field assignment parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected assignment expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 7: Expression statements wrap correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_expression_statements_parse(value in 1i64..1000) {
        let code = format!("{value}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse expression: {}", code);
        // Any expression should parse successfully
    }

    #[test]
    fn prop_method_call_expressions_parse(
        value in 1i64..1000
    ) {
        let code = format!("{value}.to_string()");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse method call: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::MethodCall { .. } = expr.kind {
                // Correct - method call parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected method call, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 8: Import/export statements parse correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_import_statements_parse(
        module_name in "[a-z][a-z0-9_]{0,10}"
    ) {
        // Skip reserved keywords
        if is_reserved_keyword(&module_name) {
            return Ok(());
        }

        let code = format!("import {module_name}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse import expression: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Import { .. } = expr.kind {
                // Correct - import expression parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected import expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 9: Struct definitions preserve fields
// ============================================================================

proptest! {
    #[test]
    fn prop_struct_definitions_parse_single_field(
        field_name in "[a-z][a-z0-9_]{0,10}"
    ) {
        // Skip reserved keywords
        if is_reserved_keyword(&field_name) {
            return Ok(());
        }

        let code = format!("struct Test {{ {field_name}: i32 }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse struct definition: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Struct { fields, .. } = &expr.kind {
                prop_assert_eq!(fields.len(), 1, "Struct should have 1 field");
            } else {
                return Err(TestCaseError::fail(format!("Expected struct definition, got {:?}", expr.kind)));
            }
        }
    }
}

#[test]
fn prop_empty_structs_parse() {
    let code = "struct Empty {}";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse empty struct");
    if let Ok(expr) = result {
        if let ExprKind::Struct { fields, .. } = &expr.kind {
            assert_eq!(fields.len(), 0, "Empty struct should have 0 fields");
        } else {
            panic!("Expected struct definition, got {:?}", expr.kind);
        }
    }
}

// ============================================================================
// Property 10: Match expressions parse arms correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_match_expressions_parse_single_arm(
        pattern_val in 1i64..100,
        arm_val in 1i64..100
    ) {
        let code = format!("match x {{ {pattern_val} => {arm_val} }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse match expression: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Match { arms, .. } = &expr.kind {
                prop_assert_eq!(arms.len(), 1, "Match should have 1 arm");
            } else {
                return Err(TestCaseError::fail(format!("Expected match expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_match_expressions_parse_multiple_arms(
        val1 in 1i64..30,
        val2 in 31i64..60,
        val3 in 61i64..100
    ) {
        let code = format!("match x {{ {val1} => {val1}, {val2} => {val2}, _ => {val3} }}");
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse match with multiple arms: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Match { arms, .. } = &expr.kind {
                prop_assert_eq!(arms.len(), 3, "Match should have 3 arms");
            } else {
                return Err(TestCaseError::fail(format!("Expected match expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if identifier is a reserved keyword
fn is_reserved_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "let"
            | "mut"
            | "fn"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
            | "match"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "pub"
            | "use"
            | "true"
            | "false"
            | "nil"
            | "null"
    )
}
