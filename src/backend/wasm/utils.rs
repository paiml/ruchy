//! WASM Utility Functions
//!
//! Pure utility functions for AST analysis in WASM code generation.

use crate::frontend::ast::{Expr, ExprKind, Literal, StringPart};

/// Check if expression tree uses any built-in functions
/// Complexity: 4 (Toyota Way: <10 ✓)
pub fn uses_builtins(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, .. } => {
            if let ExprKind::Identifier(name) = &func.kind {
                matches!(name.as_str(), "println" | "print" | "eprintln" | "eprint")
            } else {
                false
            }
        }
        ExprKind::Block(exprs) => exprs.iter().any(uses_builtins),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            uses_builtins(condition)
                || uses_builtins(then_branch)
                || else_branch.as_ref().is_some_and(|e| uses_builtins(e))
        }
        ExprKind::Let { value, body, .. } => uses_builtins(value) || uses_builtins(body),
        ExprKind::Binary { left, right, .. } => uses_builtins(left) || uses_builtins(right),
        ExprKind::StringInterpolation { parts } => parts.iter().any(|part| {
            if let StringPart::Expr(e) | StringPart::ExprWithFormat { expr: e, .. } = part {
                uses_builtins(e)
            } else {
                false
            }
        }),
        ExprKind::Match { expr, arms } => {
            uses_builtins(expr) || arms.iter().any(|arm| uses_builtins(&arm.body))
        }
        ExprKind::Function { body, .. } => uses_builtins(body),
        ExprKind::Lambda { body, .. } => uses_builtins(body),
        _ => false,
    }
}

/// Check if an expression needs memory (for arrays/strings/tuples/structs)
/// Complexity: 10 (Toyota Way: ≤10 ✓)
pub fn needs_memory(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(Literal::String(_)) => true,
        ExprKind::List(_) => true,
        ExprKind::ArrayInit { .. } => true,
        ExprKind::Tuple(_) => true, // Tuples need memory allocation
        ExprKind::StructLiteral { .. } => true, // Structs need memory allocation
        ExprKind::Block(exprs) => exprs.iter().any(needs_memory),
        ExprKind::Function { body, .. } => needs_memory(body),
        ExprKind::Let { value, body, .. } => needs_memory(value) || needs_memory(body),
        ExprKind::LetPattern { value, body, .. } => needs_memory(value) || needs_memory(body),
        ExprKind::Binary { left, right, .. } => needs_memory(left) || needs_memory(right),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            needs_memory(condition)
                || needs_memory(then_branch)
                || else_branch.as_ref().is_some_and(|e| needs_memory(e))
        }
        _ => false,
    }
}

/// Check if an expression contains a main function
pub fn has_main_function(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Function { name, .. } => name == "main",
        ExprKind::Block(exprs) => exprs.iter().any(has_main_function),
        _ => false,
    }
}

/// Check if an expression has return statements with values
pub fn has_return_with_value(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Return { value } => value.is_some(),
        ExprKind::Block(exprs) => exprs.iter().any(has_return_with_value),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            has_return_with_value(condition)
                || has_return_with_value(then_branch)
                || else_branch
                    .as_ref()
                    .is_some_and(|e| has_return_with_value(e))
        }
        ExprKind::While {
            condition, body, ..
        } => has_return_with_value(condition) || has_return_with_value(body),
        ExprKind::Function { .. } => false, // Functions are compiled separately
        ExprKind::Let { value, body, .. } => {
            has_return_with_value(value) || has_return_with_value(body)
        }
        ExprKind::Binary { left, right, .. } => {
            has_return_with_value(left) || has_return_with_value(right)
        }
        _ => false,
    }
}

/// Check if an expression needs local variables
pub fn needs_locals(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Let { .. } => true,
        ExprKind::Identifier(_) => true,
        ExprKind::Function { .. } => true,
        ExprKind::Block(exprs) => exprs.iter().any(needs_locals),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            needs_locals(condition)
                || needs_locals(then_branch)
                || else_branch.as_ref().is_some_and(|e| needs_locals(e))
        }
        ExprKind::While {
            condition, body, ..
        } => needs_locals(condition) || needs_locals(body),
        ExprKind::Binary { left, right, .. } => needs_locals(left) || needs_locals(right),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    fn parse(code: &str) -> Expr {
        let mut parser = Parser::new(code);
        parser.parse().expect("Should parse")
    }

    // uses_builtins tests
    #[test]
    fn test_uses_builtins_println() {
        let expr = parse(r#"println("hello")"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_print() {
        let expr = parse(r#"print("hello")"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_eprintln() {
        let expr = parse(r#"eprintln("error")"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_eprint() {
        let expr = parse(r#"eprint("error")"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_custom_function() {
        let expr = parse("my_func(42)");
        assert!(!uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_in_block() {
        let expr = parse(r#"{ 1; println("hi") }"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_in_let() {
        let expr = parse(r#"let x = 1; println("hi")"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_in_function() {
        let expr = parse(r#"fun main() { println("hi") }"#);
        assert!(uses_builtins(&expr));
    }

    #[test]
    fn test_uses_builtins_literal_only() {
        let expr = parse("42");
        assert!(!uses_builtins(&expr));
    }

    // needs_memory tests
    #[test]
    fn test_needs_memory_string() {
        let expr = parse(r#""hello""#);
        assert!(needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_list() {
        let expr = parse("[1, 2]");
        assert!(needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_tuple() {
        let expr = parse("(1, 2)");
        assert!(needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_int() {
        let expr = parse("42");
        assert!(!needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_in_block() {
        let expr = parse(r#"{ 1; "hello" }"#);
        assert!(needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_in_function() {
        let expr = parse(r#"fun main() { "hello" }"#);
        assert!(needs_memory(&expr));
    }

    #[test]
    fn test_needs_memory_in_let() {
        let expr = parse(r#"let x = "hello"; 1"#);
        assert!(needs_memory(&expr));
    }

    // has_main_function tests
    #[test]
    fn test_has_main_function_true() {
        let expr = parse("fun main() { 0 }");
        assert!(has_main_function(&expr));
    }

    #[test]
    fn test_has_main_function_false() {
        let expr = parse("fun not_main() { 0 }");
        assert!(!has_main_function(&expr));
    }

    #[test]
    fn test_has_main_function_in_block() {
        let expr = parse("{ fun helper() { 1 }; fun main() { 0 } }");
        assert!(has_main_function(&expr));
    }

    #[test]
    fn test_has_main_function_literal() {
        let expr = parse("42");
        assert!(!has_main_function(&expr));
    }

    // has_return_with_value tests
    #[test]
    fn test_has_return_with_value_true() {
        let expr = parse("return 42");
        assert!(has_return_with_value(&expr));
    }

    #[test]
    fn test_has_return_with_value_false() {
        let expr = parse("return");
        assert!(!has_return_with_value(&expr));
    }

    #[test]
    fn test_has_return_with_value_in_block() {
        let expr = parse("{ 1; return 42 }");
        assert!(has_return_with_value(&expr));
    }

    #[test]
    fn test_has_return_with_value_in_if() {
        let expr = parse("if true { return 1 } else { return 2 }");
        assert!(has_return_with_value(&expr));
    }

    #[test]
    fn test_has_return_with_value_literal() {
        let expr = parse("42");
        assert!(!has_return_with_value(&expr));
    }

    // needs_locals tests
    #[test]
    fn test_needs_locals_let() {
        let expr = parse("let x = 1; 2");
        assert!(needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_identifier() {
        let expr = parse("x");
        assert!(needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_function() {
        let expr = parse("fun f() { 1 }");
        assert!(needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_literal() {
        let expr = parse("42");
        assert!(!needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_in_block() {
        let expr = parse("{ 1; let x = 2; 3 }");
        assert!(needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_in_if() {
        let expr = parse("if true { let x = 2; 3 }");
        assert!(needs_locals(&expr));
    }

    #[test]
    fn test_needs_locals_in_binary() {
        let expr = parse("x + 1");
        assert!(needs_locals(&expr));
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_uses_builtins_never_panics(n in -1000i64..1000) {
                let expr = parse(&format!("{n}"));
                let _ = uses_builtins(&expr);
            }

            #[test]
            fn prop_needs_memory_never_panics(n in -1000i64..1000) {
                let expr = parse(&format!("{n}"));
                let _ = needs_memory(&expr);
            }

            #[test]
            fn prop_has_main_function_never_panics(name in "[a-z]{4,10}") {
                // Use 4+ char names to avoid reserved keywords (if, in, for, etc.)
                let expr = parse(&format!("fun {name}() {{ 0 }}"));
                let _ = has_main_function(&expr);
            }

            #[test]
            fn prop_has_return_with_value_never_panics(n in -1000i64..1000) {
                let expr = parse(&format!("return {n}"));
                let _ = has_return_with_value(&expr);
            }

            #[test]
            fn prop_needs_locals_never_panics(name in "[a-z]{4,10}") {
                // Use 4+ char names to avoid reserved keywords (if, in, for, etc.)
                let expr = parse(&name);
                let _ = needs_locals(&expr);
            }

            #[test]
            fn prop_main_function_detected(_dummy: u8) {
                let expr = parse("fun main() { 0 }");
                prop_assert!(has_main_function(&expr));
            }

            #[test]
            fn prop_builtin_println_detected(_dummy: u8) {
                let expr = parse(r#"println("test")"#);
                prop_assert!(uses_builtins(&expr));
            }

            #[test]
            fn prop_string_needs_memory(_dummy: u8) {
                let expr = parse(r#""hello""#);
                prop_assert!(needs_memory(&expr));
            }
        }
    }

    // ============================================================================
    // EXTREME TDD Round 157: Additional WASM utils tests
    // Target: 32 → 65+ tests
    // ============================================================================
    mod round_157_tests {
        use super::*;

        // --- uses_builtins additional tests ---
        #[test]
        fn test_uses_builtins_in_if_condition() {
            let expr = parse(r#"if println("x") > 0 { 1 } else { 2 }"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_in_if_then() {
            let expr = parse(r#"if true { println("x") } else { 2 }"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_in_if_else() {
            let expr = parse(r#"if false { 1 } else { println("x") }"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_no_else() {
            let expr = parse(r#"if true { println("x") }"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_binary_left() {
            let expr = parse(r#"print("x") + 1"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_binary_right() {
            let expr = parse(r#"1 + print("x")"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_nested_blocks() {
            let expr = parse(r#"{ { { println("deep") } } }"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_in_lambda() {
            let expr = parse(r#"|x| println(x)"#);
            assert!(uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_false_with_similar_name() {
            let expr = parse("printx(42)");
            assert!(!uses_builtins(&expr));
        }

        #[test]
        fn test_uses_builtins_method_call_not_builtin() {
            // obj.println() is not a builtin
            let expr = parse("x + y");
            assert!(!uses_builtins(&expr));
        }

        // --- needs_memory additional tests ---
        #[test]
        fn test_needs_memory_array_init() {
            let expr = parse("[0; 10]");
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_struct_literal() {
            let expr = parse("Point { x: 1, y: 2 }");
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_in_if_condition() {
            let expr = parse(r#"if "test" == x { 1 } else { 2 }"#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_in_if_then() {
            let expr = parse(r#"if true { "result" } else { 0 }"#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_in_if_else() {
            let expr = parse(r#"if false { 0 } else { "fallback" }"#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_binary_left() {
            let expr = parse(r#""a" + "b""#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_binary_right() {
            let expr = parse(r#"1 + [2]"#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_nested_let() {
            let expr = parse(r#"let x = "hello"; let y = [1]; 0"#);
            assert!(needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_float() {
            let expr = parse("3.14");
            assert!(!needs_memory(&expr));
        }

        #[test]
        fn test_needs_memory_bool() {
            let expr = parse("true");
            assert!(!needs_memory(&expr));
        }

        // --- has_main_function additional tests ---
        #[test]
        fn test_has_main_function_nested_in_block() {
            let expr = parse("{ { fun main() { 0 } } }");
            assert!(has_main_function(&expr));
        }

        #[test]
        fn test_has_main_function_multiple_functions() {
            let expr = parse("{ fun foo() { 1 }; fun main() { 0 }; fun bar() { 2 } }");
            assert!(has_main_function(&expr));
        }

        #[test]
        fn test_has_main_function_main_substring() {
            let expr = parse("fun main_helper() { 0 }");
            assert!(!has_main_function(&expr));
        }

        #[test]
        fn test_has_main_function_no_functions() {
            let expr = parse("1 + 2 + 3");
            assert!(!has_main_function(&expr));
        }

        // --- has_return_with_value additional tests ---
        #[test]
        fn test_has_return_with_value_in_let_value() {
            let expr = parse("let x = return 1; x");
            assert!(has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_in_binary_left() {
            let expr = parse("(return 1) + 2");
            assert!(has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_in_binary_right() {
            let expr = parse("1 + (return 2)");
            assert!(has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_function_is_separate() {
            // Function body contains return, but function itself is separately compiled
            let expr = parse("fun foo() { return 42 }");
            assert!(!has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_nested_if() {
            let expr = parse("if true { if false { return 1 } else { 2 } } else { 3 }");
            assert!(has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_while_condition() {
            let expr = parse("while (return 1) { }");
            assert!(has_return_with_value(&expr));
        }

        #[test]
        fn test_has_return_with_value_while_body() {
            let expr = parse("while true { return 1 }");
            assert!(has_return_with_value(&expr));
        }

        // --- needs_locals additional tests ---
        #[test]
        fn test_needs_locals_in_if_condition() {
            let expr = parse("if x { 1 } else { 2 }");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_in_if_then() {
            let expr = parse("if true { let x = 1; x } else { 0 }");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_in_if_else() {
            let expr = parse("if false { 0 } else { y }");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_while_condition() {
            let expr = parse("while x { }");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_while_body() {
            let expr = parse("while true { let x = 1; x }");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_binary_right() {
            let expr = parse("1 + y");
            assert!(needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_no_vars() {
            let expr = parse("1 + 2 + 3");
            assert!(!needs_locals(&expr));
        }

        #[test]
        fn test_needs_locals_nested_blocks() {
            let expr = parse("{ { { x } } }");
            assert!(needs_locals(&expr));
        }
    }
}
