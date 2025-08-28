#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args
)]

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};
use ruchy::runtime::{Repl, ReplState, ReplConfig};
use std::time::Duration;

/// Generate valid Ruchy identifiers
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}".prop_map(String::from)
}

/// Generate valid integers
fn valid_integer() -> impl Strategy<Value = i64> {
    any::<i64>()
}

/// Generate valid strings
fn valid_string() -> impl Strategy<Value = String> {
    ".*".prop_map(|s: String| s.chars().take(100).collect())
}

/// Generate simple expressions
fn simple_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        valid_integer().prop_map(|i| i.to_string()),
        valid_string().prop_map(|s| format!(r#""{}""#, s.escape_default())),
        valid_identifier(),
    ]
}

proptest! {
    /// Test that valid let statements always parse
    #[test]
    fn prop_let_statement_parses(
        var in valid_identifier(),
        value in simple_expr()
    ) {
        let input = format!("let {} = {}", var, value);
        let mut parser = Parser::new(&input);

        // Should parse without panicking
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse: {}", input);
    }

    /// Test that parsed code can be transpiled
    #[test]
    fn prop_parse_transpile_pipeline(
        var in valid_identifier(),
        value in valid_integer()
    ) {
        let input = format!("let {} = {}", var, value);
        let mut parser = Parser::new(&input);

        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            prop_assert!(result.is_ok(), "Failed to transpile: {}", input);
        }
    }

    /// Test function definitions
    #[test]
    fn prop_function_definition(
        name in valid_identifier(),
        param in valid_identifier(),
        body in valid_integer()
    ) {
        let input = format!(
            "fn {}({}: i32) -> i32 {{ {} }}",
            name, param, body
        );
        let mut parser = Parser::new(&input);

        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse function: {}", input);
    }

    /// Test binary operations maintain precedence
    #[test]
    fn prop_binary_op_precedence(
        a in 1i32..100,
        b in 1i32..100,
        c in 1i32..100
    ) {
        let input = format!("{} + {} * {}", a, b, c);
        let mut parser = Parser::new(&input);

        if let Ok(ast) = parser.parse_expr() {
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // The multiplication should have higher precedence
                // Allow various valid representations of precedence
                prop_assert!(
                    rust_str.contains(&format!("({} * {})", b, c)) ||
                    rust_str.contains(&format!("{} + {} * {}", a, b, c)) ||
                    rust_str.contains(&format!("{}i32 + {}i32 * {}i32", a, b, c)) ||
                    rust_str.contains(&format!("({} + ({} * {}))", a, b, c)) ||
                    rust_str.contains(&format!("({}i32 + ({}i32 * {}i32))", a, b, c)),
                    "Precedence not preserved in: {}", rust_code
                );
            }
        }
    }

    /// Test string literals are properly escaped
    #[test]
    fn prop_string_escaping(s in ".*") {
        let input = format!(r#"let x = "{}""#, s.escape_default());
        let mut parser = Parser::new(&input);

        // Should handle any string content
        let result = parser.parse();
        if result.is_ok() {
            let transpiler = Transpiler::new();
            let ast = result.unwrap();
            let transpiled = transpiler.transpile(&ast);
            prop_assert!(transpiled.is_ok(), "Failed to transpile string: {:?}", s);
        }
    }

    // REPL Property Tests (REPL-TEST-003)
    
    /// Test that REPL evaluation preserves type safety
    #[test]
    fn prop_repl_type_safety(input in simple_expr()) {
        let mut repl = Repl::new().unwrap();
        
        match repl.eval(&input) {
            Ok(_) => {
                // Success should maintain Ready state
                prop_assert!(matches!(repl.get_state(), ReplState::Ready));
                prop_assert!(!repl.is_failed());
            }
            Err(_) => {
                // Errors are acceptable, should not corrupt state
                prop_assert!(!repl.is_failed() || repl.recover().is_ok());
            }
        }
    }

    /// Test REPL state transitions are valid
    #[test] 
    fn prop_repl_state_transitions(
        setup in prop::collection::vec(simple_expr(), 0..5),
        operations in prop::collection::vec(0u8..4, 0..10)
    ) {
        let mut repl = Repl::new().unwrap();
        let mut checkpoint = None;
        
        // Setup phase
        for expr in &setup {
            let _ = repl.eval(expr);
        }
        
        // Test operations
        for op in operations {
            let prev_valid = matches!(repl.get_state(), ReplState::Ready) || repl.is_failed();
            
            match op % 4 {
                0 => { let _ = repl.eval("1 + 1"); }
                1 => { checkpoint = Some(repl.checkpoint()); }
                2 => { if repl.is_failed() { let _ = repl.recover(); } }
                _ => { if let Some(cp) = &checkpoint { repl.restore_checkpoint(cp); } }
            }
            
            let curr_valid = matches!(repl.get_state(), ReplState::Ready) || repl.is_failed();
            prop_assert!(prev_valid || curr_valid, "Invalid state transition detected");
        }
    }

    /// Test memory bounds are respected
    #[test]
    fn prop_memory_bounds(inputs in prop::collection::vec(simple_expr(), 0..10)) {
        let config = ReplConfig {
            max_memory: 1024 * 1024, // 1MB
            timeout: Duration::from_millis(100),
            max_depth: 100,
            debug: false,
        };
        
        let mut repl = Repl::with_config(config).unwrap();
        
        for input in inputs {
            let _ = repl.eval(&input);
            prop_assert!(repl.memory_used() <= 1024 * 1024, "Memory bound exceeded");
        }
    }

    /// Test deterministic evaluation
    #[test]
    fn prop_deterministic_evaluation(input in simple_expr()) {
        let mut repl1 = Repl::new().unwrap();
        let mut repl2 = Repl::new().unwrap();
        
        let result1 = repl1.eval(&input);
        let result2 = repl2.eval(&input);
        
        match (result1, result2) {
            (Ok(val1), Ok(val2)) => prop_assert_eq!(val1.trim(), val2.trim()),
            (Err(_), Err(_)) => {}, // Both failing is fine
            _ => prop_assert!(false, "Non-deterministic evaluation detected"),
        }
    }

    /// Test checkpoint consistency
    #[test]
    fn prop_checkpoint_consistency(
        setup in prop::collection::vec(simple_expr(), 0..5),
        changes in prop::collection::vec(simple_expr(), 0..5)
    ) {
        let mut repl = Repl::new().unwrap();
        
        // Setup initial state and record results
        let mut setup_results = Vec::new();
        for expr in &setup {
            if let Ok(result) = repl.eval(expr) {
                setup_results.push((expr.clone(), result));
            }
        }
        
        let checkpoint = repl.checkpoint();
        
        // Make changes
        for expr in &changes {
            let _ = repl.eval(expr);
        }
        
        // Restore checkpoint
        repl.restore_checkpoint(&checkpoint);
        
        // Verify setup expressions give same results
        for (expr, expected) in setup_results {
            if let Ok(actual) = repl.eval(&expr) {
                prop_assert_eq!(actual.trim(), expected.trim(), 
                    "Checkpoint restore inconsistency for: {}", expr);
            }
        }
    }

    /// Test error recovery preserves validity
    #[test]
    fn prop_error_recovery(setup_expr in simple_expr()) {
        let mut repl = Repl::new().unwrap();
        
        // Setup valid state
        let _ = repl.eval(&setup_expr);
        let checkpoint = repl.checkpoint();
        
        // Force failed state for testing
        repl.set_state_for_testing(ReplState::Failed(checkpoint));
        prop_assert!(repl.is_failed());
        
        // Recovery should restore validity
        let recovery = repl.recover();
        prop_assert!(recovery.is_ok(), "Recovery failed");
        prop_assert!(matches!(repl.get_state(), ReplState::Ready));
        prop_assert!(!repl.is_failed());
        
        // Should be able to evaluate after recovery
        let test_eval = repl.eval("1 + 1");
        prop_assert!(test_eval.is_ok(), "Cannot evaluate after recovery");
    }

    /// Test resource bounds never exceeded
    #[test]
    fn prop_resource_bounds(inputs in prop::collection::vec(simple_expr(), 0..8)) {
        let config = ReplConfig {
            max_memory: 512 * 1024, // 512KB
            timeout: Duration::from_millis(50),
            max_depth: 50,
            debug: false,
        };
        
        let mut repl = Repl::with_config(config).unwrap();
        
        for input in inputs {
            let start = std::time::Instant::now();
            let _ = repl.eval(&input);
            let elapsed = start.elapsed();
            
            // Allow some overhead but enforce reasonable bounds
            prop_assert!(elapsed < Duration::from_millis(200), 
                "Evaluation took too long: {:?} for input: {}", elapsed, input);
            prop_assert!(repl.memory_used() <= 512 * 1024,
                "Memory usage exceeded bounds: {} bytes", repl.memory_used());
        }
    }
}
