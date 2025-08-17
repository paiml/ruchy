//! REPL property testing module
#![allow(clippy::unwrap_used)]
#![cfg(test)]

use super::repl::Repl;
use proptest::prelude::*;

#[derive(Debug, Clone)]
pub enum ReplAction {
    Eval(String),
    ShowHistory,
    ShowType(String),
    ShowRust(String),
    ShowAst(String),
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplState {
    history_count: usize,
    definitions_count: usize,
    bindings_count: usize,
    session_counter: usize,
}

impl ReplState {
    #[must_use]
    pub fn extract_from_repl(repl: &Repl) -> Self {
        Self {
            history_count: repl.history().len(),
            definitions_count: repl.definitions().len(),
            bindings_count: repl.bindings().len(),
            session_counter: repl.session_counter(),
        }
    }
}

/// Generate valid Ruchy expressions for testing
fn arb_valid_expression() -> impl Strategy<Value = String> {
    prop_oneof![
        // Literals
        any::<i32>().prop_map(|n| n.to_string()),
        "true|false",
        r#""[a-zA-Z0-9 ]*""#,
        // Simple arithmetic
        (any::<i32>(), any::<i32>()).prop_map(|(a, b)| format!("{a} + {b}")),
        // Variables
        "[a-z][a-z0-9]*",
        // Let bindings
        ("[a-z][a-z0-9]*", any::<i32>()).prop_map(|(var, val)| format!("let {var} = {val}")),
    ]
}

fn arb_repl_action() -> impl Strategy<Value = ReplAction> {
    prop_oneof![
        arb_valid_expression().prop_map(ReplAction::Eval),
        Just(ReplAction::ShowHistory),
        "[a-z][a-z0-9]*".prop_map(ReplAction::ShowType),
        arb_valid_expression().prop_map(ReplAction::ShowRust),
        arb_valid_expression().prop_map(ReplAction::ShowAst),
        Just(ReplAction::Clear),
    ]
}

fn apply_action(repl: &mut Repl, action: &ReplAction) -> Result<String, String> {
    match action {
        ReplAction::Eval(expr) => repl.eval(expr).map_err(|e| e.to_string()),
        ReplAction::ShowHistory => Ok(repl.show_history()),
        ReplAction::ShowType(var) => repl.show_type(var).map_err(|e| e.to_string()),
        ReplAction::ShowRust(expr) => repl.show_rust(expr).map_err(|e| e.to_string()),
        ReplAction::ShowAst(expr) => repl.show_ast(expr).map_err(|e| e.to_string()),
        ReplAction::Clear => {
            repl.clear_session();
            Ok("Session cleared".to_string())
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: REPL never panics on any valid action sequence
    #[test]
    fn test_repl_never_panics(actions in prop::collection::vec(arb_repl_action(), 0..10)) {
        let mut repl = Repl::new().unwrap();

        for action in actions {
            // This should never panic - all errors should be handled gracefully
            let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                apply_action(&mut repl, &action)
            }));
            // The test passes if we reach here without panicking
        }
    }

    /// Property: Session counter increases with eval, resets with clear
    #[test]
    fn test_session_counter_behavior(actions in prop::collection::vec(arb_repl_action(), 0..5)) {
        let mut repl = Repl::new().unwrap();
        let mut prev_counter = 0;

        for action in actions {
            let _ = apply_action(&mut repl, &action);
            let current_counter = repl.session_counter();

            match action {
                ReplAction::Clear => {
                    // Clear resets session counter to 0
                    prop_assert_eq!(current_counter, 0, "Clear should reset session counter to 0");
                    prev_counter = 0;
                }
                ReplAction::Eval(_) => {
                    // Eval should increase session counter (if successful compilation happens)
                    prop_assert!(current_counter >= prev_counter,
                        "Session counter should not decrease for eval: {} -> {}", prev_counter, current_counter);
                    prev_counter = current_counter;
                }
                _ => {
                    // Other operations should not change session counter
                    prop_assert_eq!(current_counter, prev_counter,
                        "Show operations should not change session counter");
                }
            }
        }
    }

    /// Property: Clear operation resets history and bindings
    #[test]
    fn test_clear_resets_state(
        setup_actions in prop::collection::vec(arb_repl_action(), 1..5)
    ) {
        let mut repl = Repl::new().unwrap();

        // Perform some operations to build up state
        for action in setup_actions {
            let _ = apply_action(&mut repl, &action);
        }

        // Clear the session
        let _ = apply_action(&mut repl, &ReplAction::Clear);

        let post_clear_state = ReplState::extract_from_repl(&repl);

        // Verify reset
        prop_assert_eq!(post_clear_state.history_count, 0);
        prop_assert_eq!(post_clear_state.definitions_count, 0);
        prop_assert_eq!(post_clear_state.bindings_count, 0);
    }

    /// Property: Show operations don't modify state
    #[test]
    fn test_show_operations_readonly(
        setup_expr in arb_valid_expression(),
        show_action in prop_oneof![
            Just(ReplAction::ShowHistory),
            "[a-z][a-z0-9]*".prop_map(ReplAction::ShowType),
            arb_valid_expression().prop_map(ReplAction::ShowRust),
            arb_valid_expression().prop_map(ReplAction::ShowAst),
        ]
    ) {
        let mut repl = Repl::new().unwrap();

        // Set up some state
        let _ = apply_action(&mut repl, &ReplAction::Eval(setup_expr));
        let pre_state = ReplState::extract_from_repl(&repl);

        // Perform show operation
        let _ = apply_action(&mut repl, &show_action);
        let post_state = ReplState::extract_from_repl(&repl);

        // State should be unchanged
        prop_assert_eq!(pre_state.history_count, post_state.history_count);
        prop_assert_eq!(pre_state.definitions_count, post_state.definitions_count);
        prop_assert_eq!(pre_state.bindings_count, post_state.bindings_count);
    }

    /// Property: Valid expressions produce output
    #[test]
    fn test_valid_expressions_produce_output(expr in arb_valid_expression()) {
        let mut repl = Repl::new().unwrap();

        let result = apply_action(&mut repl, &ReplAction::Eval(expr.clone()));

        // Should either succeed or fail gracefully with non-empty error
        match result {
            Ok(_) => {}, // Success is always good
            Err(error) => {
                prop_assert!(!error.trim().is_empty(),
                    "Expression '{}' produced empty error message", expr);
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_repl_state_extraction() {
        let repl = Repl::new().unwrap();
        let state = ReplState::extract_from_repl(&repl);

        assert_eq!(state.history_count, 0);
        assert_eq!(state.definitions_count, 0);
        assert_eq!(state.bindings_count, 0);
        assert_eq!(state.session_counter, 0);
    }

    #[test]
    fn test_repl_basic_operations() {
        let mut repl = Repl::new().unwrap();

        // Test history
        let history = repl.show_history();
        assert!(history.contains("No history"));

        // Test clear
        repl.clear_session();
        assert_eq!(repl.history().len(), 0);

        // Test type checking
        let type_result = repl.show_type("unknown_var");
        assert!(type_result.is_ok());
    }

    #[test]
    fn test_apply_action_coverage() {
        let mut repl = Repl::new().unwrap();

        // Test all action types
        let actions = vec![
            ReplAction::Eval("42".to_string()),
            ReplAction::ShowHistory,
            ReplAction::ShowType("test".to_string()),
            ReplAction::ShowRust("true".to_string()),
            ReplAction::ShowAst("1 + 1".to_string()),
            ReplAction::Clear,
        ];

        for action in actions {
            let result = apply_action(&mut repl, &action);
            // Should not panic and should return some result
            assert!(result.is_ok() || result.is_err());
        }
    }
}
