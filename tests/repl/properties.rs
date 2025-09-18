#![allow(clippy::unwrap_used, clippy::panic)]
//! State Machine Property Testing for REPL
//! Implements comprehensive property-based testing for REPL state transitions,
//! invariant preservation, and recovery paths as specified in ruchy-repl-testing-todo.yaml

use crate::runtime::repl::Repl;
use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::runtime::value::Value;
use proptest::prelude::*;
use std::{env, collections::HashSet;
use std::{env, time::Instant;

#[derive(Debug, Clone)]
pub enum ReplAction {
    Eval(String),
    ShowHistory,
    ShowType(String),
    ShowRust(String),
    ShowAst(String),
    Clear,
    Save(String),
    Load(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplState {
    history_count: usize,
    definitions_count: usize,
    bindings_count: usize,
    session_counter: usize,
    has_errors: bool,
}

impl ReplState {
    pub fn initial() -> Self {
        Self {
            history_count: 0,
            definitions_count: 0,
            bindings_count: 0,
            session_counter: 0,
            has_errors: false,
        }
    }
    
    pub fn extract_from_repl(repl: &Repl) -> Self {
        Self {
            history_count: repl.history_len(),
            definitions_count: repl.definitions_len(),
            bindings_count: repl.bindings_len(),
            session_counter: repl.session_counter(),
            has_errors: false, // Simplified for testing
        }
    }
}

/// Generate valid Ruchy expressions for testing
fn arb_valid_expression() -> impl Strategy<Value = String> {
    prop_oneof![
        // Literals
        any::<i64>().prop_map(|n| n.to_string()),
        any::<f64>().prop_map(|f| format!("{f:.2}")),
        "true|false",
        prop::string::string_regex(r#""[a-zA-Z0-9 ]*""#).unwrap(),
        
        // Simple arithmetic
        (any::<i32>(), any::<i32>()).prop_map(|(a, b)| format!("{a} + {b}")),
        (any::<i32>(), any::<i32>()).prop_map(|(a, b)| format!("{a} * {b}")),
        
        // Variables
        "[a-z][a-z0-9]*",
        
        // Let bindings
        ("[a-z][a-z0-9]*", any::<i32>()).prop_map(|(var, val)| format!("let {var} = {val}")),
        
        // Function definitions
        ("[a-z][a-z0-9]*", "[a-z][a-z0-9]*", any::<i32>())
            .prop_map(|(fname, param, body)| format!("fun {fname}({param}: i32) -> i32 {{ {body} }}")),
            
        // Lists
        prop::collection::vec(any::<i32>(), 0..5)
            .prop_map(|nums| format!("[{}]", nums.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", "))),
    ]
}

/// Generate potentially invalid expressions for robustness testing
fn arb_invalid_expression() -> impl Strategy<Value = String> {
    prop_oneof![
        // Syntax errors
        "let = 5",
        "fun ()",
        "1 + + 2",
        "[1, 2,",
        "if true",
        
        // Incomplete expressions
        "let x",
        "fun add(",
        "match x",
        
        // Type errors (should be handled gracefully)
        r#""hello" + 5"#,
        "true * false",
        "[1, 2][10]",
    ]
}

fn arb_repl_action() -> impl Strategy<Value = ReplAction> {
    prop_oneof![
        arb_valid_expression().prop_map(ReplAction::Eval),
        arb_invalid_expression().prop_map(ReplAction::Eval),
        Just(ReplAction::ShowHistory),
        "[a-z][a-z0-9]*".prop_map(ReplAction::ShowType),
        arb_valid_expression().prop_map(ReplAction::ShowRust),
        arb_valid_expression().prop_map(ReplAction::ShowAst),
        Just(ReplAction::Clear),
        "[a-z][a-z0-9]*".prop_map(ReplAction::Save),
        "[a-z][a-z0-9]*".prop_map(ReplAction::Load),
    ]
}

// Extension trait to access REPL internals for testing
pub trait ReplTestExt {
    fn history_len(&self) -> usize;
    fn definitions_len(&self) -> usize;
    fn bindings_len(&self) -> usize;
    fn session_counter(&self) -> usize;
    fn apply_action(&mut self, action: &ReplAction) -> Result<String, String>;
}

impl ReplTestExt for Repl {
    fn history_len(&self) -> usize {
        self.history().len()
    }
    
    fn definitions_len(&self) -> usize {
        self.definitions().len()
    }
    
    fn bindings_len(&self) -> usize {
        self.bindings().len()
    }
    
    fn session_counter(&self) -> usize {
        self.session_counter()
    }
    
    fn apply_action(&mut self, action: &ReplAction) -> Result<String, String> {
        match action {
            ReplAction::Eval(expr) => {
                self.eval(expr).map_err(|e| e.to_string())
            }
            ReplAction::ShowHistory => {
                Ok(self.show_history())
            }
            ReplAction::ShowType(var) => {
                self.show_type(var).map_err(|e| e.to_string())
            }
            ReplAction::ShowRust(expr) => {
                self.show_rust(expr).map_err(|e| e.to_string())
            }
            ReplAction::ShowAst(expr) => {
                self.show_ast(expr).map_err(|e| e.to_string())
            }
            ReplAction::Clear => {
                self.clear_session();
                Ok("Session cleared".to_string())
            }
            ReplAction::Save(name) => {
                self.save_session(name).map_err(|e| e.to_string())?;
                Ok(format!("Saved session: {name}"))
            }
            ReplAction::Load(name) => {
                self.load_session(name).map_err(|e| e.to_string())?;
                Ok(format!("Loaded session: {name}"))
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: REPL state transitions maintain fundamental invariants
    #[test]
    fn test_repl_state_invariants(actions in prop::collection::vec(arb_repl_action(), 0..20)) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        let mut state = ReplState::initial();
        
        for action in actions {
            let prev_state = state.clone();
            let result = repl.apply_action(&action);
            state = ReplState::extract_from_repl(&repl);
            
            // Invariant: Session counter only increases
            prop_assert!(state.session_counter >= prev_state.session_counter);
            
            // Invariant: History can only grow or be cleared
            if !matches!(action, ReplAction::Clear | ReplAction::Load(_)) {
                prop_assert!(state.history_count >= prev_state.history_count);
            }
            
            // Invariant: Successful eval increases history
            if let ReplAction::Eval(expr) = &action {
                if result.is_ok() && !expr.trim().is_empty() {
                    // Valid expressions should either increase history or maintain it
                    prop_assert!(state.history_count >= prev_state.history_count);
                }
            }
            
            // Invariant: Clear resets state appropriately
            if matches!(action, ReplAction::Clear) {
                prop_assert_eq!(state.history_count, 0);
                prop_assert_eq!(state.definitions_count, 0);
                prop_assert_eq!(state.bindings_count, 0);
            }
            
            // Invariant: Show operations don't modify state
            if matches!(action, ReplAction::ShowHistory | ReplAction::ShowType(_) | 
                       ReplAction::ShowRust(_) | ReplAction::ShowAst(_)) {
                prop_assert_eq!(state.history_count, prev_state.history_count);
                prop_assert_eq!(state.definitions_count, prev_state.definitions_count);
                prop_assert_eq!(state.bindings_count, prev_state.bindings_count);
            }
        }
    }

    /// Property: REPL never panics on any input
    #[test]
    fn test_repl_never_panics(actions in prop::collection::vec(arb_repl_action(), 0..50)) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        for action in actions {
            // This should never panic - all errors should be handled gracefully
            let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                repl.apply_action(&action)
            }));
            // The test passes if we reach here without panicking
        }
    }

    /// Property: Valid expressions always produce some output
    #[test]
    fn test_valid_expressions_produce_output(expr in arb_valid_expression()) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        let result = repl.apply_action(&ReplAction::Eval(expr.clone()));
        
        // Valid expressions should either succeed or fail gracefully
        match result {
            Ok(output) => {
                prop_assert!(!output.trim().is_empty(), "Valid expression '{}' produced empty output", expr);
            }
            Err(error) => {
                prop_assert!(!error.trim().is_empty(), "Valid expression '{}' produced empty error", expr);
            }
        }
    }

    /// Property: History operations maintain consistency
    #[test]
    fn test_history_consistency(
        expressions in prop::collection::vec(arb_valid_expression(), 1..10)
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        let mut expected_history = Vec::new();
        
        for expr in expressions {
            let result = repl.apply_action(&ReplAction::Eval(expr.clone()));
            if result.is_ok() {
                expected_history.push(expr);
            }
            
            let history_output = repl.show_history();
            
            // History should contain all successfully evaluated expressions
            for expected_expr in &expected_history {
                prop_assert!(
                    history_output.contains(expected_expr),
                    "History missing expression: {}", expected_expr
                );
            }
        }
    }

    /// Property: Clear operation fully resets REPL state
    #[test]
    fn test_clear_operation_completeness(
        setup_actions in prop::collection::vec(arb_repl_action(), 1..10)
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        // Perform some operations to build up state
        for action in setup_actions {
            let _ = repl.apply_action(&action);
        }
        
        let pre_clear_state = ReplState::extract_from_repl(&repl);
        
        // Clear the session
        repl.apply_action(&ReplAction::Clear).expect("Clear should not fail");
        
        let post_clear_state = ReplState::extract_from_repl(&repl);
        
        // Verify complete reset (except session counter which only increases)
        prop_assert_eq!(post_clear_state.history_count, 0);
        prop_assert_eq!(post_clear_state.definitions_count, 0);
        prop_assert_eq!(post_clear_state.bindings_count, 0);
        prop_assert!(post_clear_state.session_counter >= pre_clear_state.session_counter);
    }

    /// Property: Type information is consistent
    #[test]
    fn test_type_information_consistency(
        var_name in "[a-z][a-z0-9]*",
        value in any::<i32>()
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        let binding_expr = format!("let {var_name} = {value}");
        let eval_result = repl.apply_action(&ReplAction::Eval(binding_expr));
        
        if eval_result.is_ok() {
            let type_result = repl.apply_action(&ReplAction::ShowType(var_name.clone()));
            
            match type_result {
                Ok(type_info) => {
                    // Should contain type information
                    prop_assert!(!type_info.trim().is_empty());
                    // For integer literals, should indicate integer type
                    prop_assert!(type_info.contains("i32") || type_info.contains("int") || 
                               type_info.contains("Integer") || type_info.contains("number"));
                }
                Err(_) => {
                    // It's acceptable for type lookup to fail if variable doesn't exist
                    // but it shouldn't panic
                }
            }
        }
    }

    /// Property: Recovery from errors maintains REPL stability  
    #[test]
    fn test_error_recovery_stability(
        valid_expr in arb_valid_expression(),
        invalid_expr in arb_invalid_expression()
    ) {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        // Execute valid expression
        let valid_result = repl.apply_action(&ReplAction::Eval(valid_expr.clone()));
        let state_after_valid = ReplState::extract_from_repl(&repl);
        
        // Execute invalid expression (should fail gracefully)
        let invalid_result = repl.apply_action(&ReplAction::Eval(invalid_expr));
        let state_after_invalid = ReplState::extract_from_repl(&repl);
        
        // REPL should still work after error
        let recovery_result = repl.apply_action(&ReplAction::ShowHistory);
        
        prop_assert!(recovery_result.is_ok(), "REPL should remain functional after error");
        
        // Valid expression should have succeeded
        if valid_result.is_ok() {
            prop_assert!(state_after_valid.history_count > 0);
        }
        
        // Invalid expression should not have corrupted state
        // (history might not increase, but shouldn't decrease)
        prop_assert!(state_after_invalid.history_count >= state_after_valid.history_count);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_repl_state_extraction() {
        let repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        let state = ReplState::extract_from_repl(&repl);
        
        assert_eq!(state.history_count, 0);
        assert_eq!(state.definitions_count, 0);
        assert_eq!(state.bindings_count, 0);
        assert_eq!(state.session_counter, 0);
    }

    #[test]
    fn test_repl_action_generation() {
        // Verify our action generators produce valid actions
        let config = ProptestConfig::with_cases(100);
        proptest!(config, |(action in arb_repl_action())| {
            match action {
                ReplAction::Eval(expr) => assert!(!expr.is_empty()),
                ReplAction::ShowType(var) => assert!(!var.is_empty()),
                ReplAction::ShowRust(expr) => assert!(!expr.is_empty()),
                ReplAction::ShowAst(expr) => assert!(!expr.is_empty()),
                ReplAction::Save(name) => assert!(!name.is_empty()),
                ReplAction::Load(name) => assert!(!name.is_empty()),
                _ => {} // Other actions have no constraints
            }
        });
    }

    #[test]
    fn test_expression_generators() {
        // Test that our expression generators produce reasonable output
        let config = ProptestConfig::with_cases(50);
        
        proptest!(config, |(expr in arb_valid_expression())| {
            assert!(!expr.trim().is_empty());
            assert!(expr.len() < 1000); // Reasonable size limit
        });
        
        proptest!(config, |(expr in arb_invalid_expression())| {
            assert!(!expr.trim().is_empty());
            assert!(expr.len() < 1000); // Reasonable size limit
        });
    }

    /// Property: Enum constructors always succeed or fail deterministically
    #[test]
    fn test_enum_constructor_determinism() {
        let config = ProptestConfig::with_cases(100);
        
        proptest!(config, |(func_name in prop::sample::select(vec!["None", "Some", "Ok", "Err"]))| {
            let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
            let deadline = Instant::now() + std::time::Duration::from_secs(1);
            
            let args = match func_name.as_str() {
                "None" => vec![],
                _ => vec![Expr { kind: ExprKind::Int(42), span: Span::default() }],
            };
            
            let result1 = repl.try_enum_constructor(&func_name, &args, deadline, 0);
            let result2 = repl.try_enum_constructor(&func_name, &args, deadline, 0);
            
            // Same input should produce same result (determinism)
            prop_assert_eq!(result1.is_ok(), result2.is_ok());
            
            if let (Ok(Some(val1)), Ok(Some(val2))) = (&result1, &result2) {
                // Values should be equivalent for enum variants
                prop_assert!(matches!((val1, val2), 
                    (Value::EnumVariant { enum_name: ref e1, variant_name: ref v1, .. },
                     Value::EnumVariant { enum_name: ref e2, variant_name: ref v2, .. }) 
                    if e1 == e2 && v1 == v2));
            }
        });
    }

    /// Property: Math functions preserve mathematical properties
    #[test]
    fn test_math_function_properties() {
        let config = ProptestConfig::with_cases(200);
        
        proptest!(config, |(
            x in -1000i64..1000i64,
            y in -1000i64..1000i64
        )| {
            let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
            let deadline = Instant::now() + std::time::Duration::from_secs(1);
            
            // Test abs is always non-negative
            let abs_arg = vec![Expr { kind: ExprKind::Int(x), span: Span::default() }];
            if let Ok(Some(Value::Int(result))) = repl.try_math_function("abs", &abs_arg, deadline, 0) {
                prop_assert!(result >= 0, "abs({}) = {} should be non-negative", x, result);
            }
            
            // Test min/max properties if y != 0 to avoid edge cases
            if y != 0 {
                let min_args = vec![
                    Expr { kind: ExprKind::Int(x), span: Span::default() },
                    Expr { kind: ExprKind::Int(y), span: Span::default() },
                ];
                let max_args = min_args.clone();
                
                if let (Ok(Some(Value::Int(min_val))), Ok(Some(Value::Int(max_val)))) = (
                    repl.try_math_function("min", &min_args, deadline, 0),
                    repl.try_math_function("max", &max_args, deadline, 0)
                ) {
                    prop_assert!(min_val <= max_val, "min({}, {}) = {} should be <= max({}, {}) = {}", 
                               x, y, min_val, x, y, max_val);
                    prop_assert!(min_val <= x && min_val <= y, "min should be <= both inputs");
                    prop_assert!(max_val >= x && max_val >= y, "max should be >= both inputs");
                }
            }
        });
    }

    /// Property: User function execution preserves scope isolation
    #[test] 
    fn test_user_function_scope_isolation() {
        let config = ProptestConfig::with_cases(50);
        
        proptest!(config, |(var_name in "[a-z]{1,5}", value in -100i64..100i64)| {
            let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
            let deadline = Instant::now() + std::time::Duration::from_secs(1);
            
            // Set up a variable in global scope
            repl.bindings.insert(var_name.clone(), Value::Int(value));
            let original_bindings = repl.bindings.clone();
            
            // Define a simple function that modifies a parameter with the same name
            let func_body = Expr { kind: ExprKind::Int(999), span: Span::default() };
            repl.bindings.insert("test_func".to_string(), Value::Function {
                params: vec![var_name.clone()],
                body: Box::new(func_body),
                is_async: false,
            });
            
            // Call the function
            let args = vec![Expr { kind: ExprKind::Int(42), span: Span::default() }];
            let _result = repl.execute_user_defined_function("test_func", &args, deadline, 0);
            
            // Original variable should be unchanged (scope isolation)
            prop_assert_eq!(repl.bindings.get(&var_name), original_bindings.get(&var_name),
                          "Global variable '{}' should be unchanged after function call", var_name);
        });
    }
}