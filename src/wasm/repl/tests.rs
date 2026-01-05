//! Tests for WasmRepl
//!
//! Comprehensive tests including unit, integration, and property tests.

use super::wasm_repl::WasmRepl;
use crate::runtime::Value;
use crate::wasm::output::ReplOutput;

#[test]
fn test_wasm_repl_creation() {
    let repl = WasmRepl::new();
    assert!(repl.is_ok());
}

#[test]
fn test_wasm_repl_default() {
    let repl = WasmRepl::default();
    assert!(repl.session_id().starts_with("session-"));
}

#[test]
fn test_session_id() {
    let repl = WasmRepl::new().expect("repl");
    assert!(repl.session_id().starts_with("session-"));
}

#[test]
fn test_session_id_unique() {
    let repl1 = WasmRepl::new().expect("repl1");
    std::thread::sleep(std::time::Duration::from_millis(1));
    let repl2 = WasmRepl::new().expect("repl2");
    assert_ne!(repl1.session_id(), repl2.session_id());
}

#[test]
fn test_eval_simple_integer() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("42").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("42".to_string()));
}

#[test]
fn test_eval_simple_float() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("3.14").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
}

#[test]
fn test_eval_simple_bool() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("true").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("true".to_string()));
}

#[test]
fn test_eval_arithmetic() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("1 + 2 * 3").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("7".to_string()));
}

#[test]
fn test_eval_parse_error() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("let = invalid").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(!output.success);
    assert!(output.error.is_some());
    assert!(output.error.unwrap().contains("Parse error"));
}

#[test]
fn test_eval_adds_to_history() {
    let mut repl = WasmRepl::new().expect("repl");
    let _ = repl.eval("1");
    let _ = repl.eval("2");
    let _ = repl.eval("3");
    let history = repl.get_history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0], "1");
    assert_eq!(history[1], "2");
    assert_eq!(history[2], "3");
}

#[test]
fn test_clear_history() {
    let mut repl = WasmRepl::new().expect("repl");
    let _ = repl.eval("1");
    let _ = repl.eval("2");
    assert_eq!(repl.get_history().len(), 2);
    repl.clear();
    assert!(repl.get_history().is_empty());
}

#[test]
fn test_clear_bindings() {
    let mut repl = WasmRepl::new().expect("repl");
    repl.bindings.insert("x".to_string(), "10".to_string());
    assert!(!repl.bindings.is_empty());
    repl.clear();
    assert!(repl.bindings.is_empty());
}

#[test]
fn test_format_value_string() {
    use std::sync::Arc;
    let value = Value::String(Arc::from("hello"));
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "hello");
}

#[test]
fn test_format_value_nil() {
    let value = Value::Nil;
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "nil");
}

#[test]
fn test_format_value_integer() {
    let value = Value::Integer(42);
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "42");
}

#[test]
fn test_format_value_bool() {
    let value = Value::Bool(true);
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "true");
}

#[test]
fn test_format_value_float() {
    let value = Value::Float(3.14);
    let result = WasmRepl::format_value_for_display(&value);
    assert!(result.contains("3.14"));
}

#[test]
fn test_eval_complex_expression() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("(1 + 2) * (3 + 4)").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("21".to_string()));
}

#[test]
fn test_eval_string_literal() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval(r#""hello world""#).expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("hello world".to_string()));
}

#[test]
fn test_eval_array_literal() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("[1, 2, 3]").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
}

#[test]
fn test_timing_info_present() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("42").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.timing.total_ms >= 0.0);
    assert!(output.timing.parse_ms >= 0.0);
    assert!(output.timing.eval_ms >= 0.0);
}

#[test]
fn test_eval_boolean_false() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("false").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("false".to_string()));
}

#[test]
fn test_eval_comparison() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("5 > 3").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("true".to_string()));
}

#[test]
fn test_eval_logical_and() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("true && false").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("false".to_string()));
}

#[test]
fn test_eval_logical_or() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("true || false").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("true".to_string()));
}

#[test]
fn test_history_accumulation() {
    let mut repl = WasmRepl::new().expect("repl");
    for i in 1..=10 {
        let _ = repl.eval(&format!("{i}"));
    }
    let history = repl.get_history();
    assert_eq!(history.len(), 10);
}

// Additional tests for edge cases
#[test]
fn test_eval_negative_number() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("-42").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("-42".to_string()));
}

#[test]
fn test_eval_zero() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("0").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("0".to_string()));
}

#[test]
fn test_eval_subtraction() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("10 - 3").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("7".to_string()));
}

#[test]
fn test_eval_division() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("20 / 4").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("5".to_string()));
}

#[test]
fn test_eval_modulo() {
    let mut repl = WasmRepl::new().expect("repl");
    let result = repl.eval("17 % 5").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(output.display, Some("2".to_string()));
}

#[test]
fn test_format_value_negative_integer() {
    let value = Value::Integer(-100);
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "-100");
}

#[test]
fn test_format_value_false() {
    let value = Value::Bool(false);
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "false");
}

#[test]
fn test_format_value_empty_string() {
    use std::sync::Arc;
    let value = Value::String(Arc::from(""));
    let result = WasmRepl::format_value_for_display(&value);
    assert_eq!(result, "");
}

#[test]
fn test_multiple_clear_calls() {
    let mut repl = WasmRepl::new().expect("repl");
    let _ = repl.eval("1");
    repl.clear();
    repl.clear(); // Second clear should not panic
    assert!(repl.get_history().is_empty());
}

#[test]
fn test_eval_after_clear() {
    let mut repl = WasmRepl::new().expect("repl");
    let _ = repl.eval("1");
    repl.clear();
    let result = repl.eval("42").expect("eval");
    let output: ReplOutput = serde_json::from_str(&result).expect("parse");
    assert!(output.success);
    assert_eq!(repl.get_history().len(), 1);
}

// Property tests
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_wasm_repl_new_never_panics(_dummy: u8) {
            let repl = WasmRepl::new();
            prop_assert!(repl.is_ok());
        }

        #[test]
        fn prop_eval_simple_never_panics(x in -1000i64..1000) {
            let mut repl = WasmRepl::new().unwrap();
            let code = format!("{x}");
            let _ = repl.eval(&code);
        }

        #[test]
        fn prop_eval_arithmetic_never_panics(
            a in -100i64..100,
            b in 1i64..100
        ) {
            let mut repl = WasmRepl::new().unwrap();
            let _ = repl.eval(&format!("{a} + {b}"));
            let _ = repl.eval(&format!("{a} - {b}"));
            let _ = repl.eval(&format!("{a} * {b}"));
            let _ = repl.eval(&format!("{a} / {b}"));
        }

        #[test]
        fn prop_get_history_valid(
            code1 in "[a-z0-9 +\\-*/]{1,20}",
            code2 in "[a-z0-9 +\\-*/]{1,20}"
        ) {
            let mut repl = WasmRepl::new().unwrap();
            let _ = repl.eval(&code1);
            let _ = repl.eval(&code2);
            let history = repl.get_history();
            prop_assert!(history.len() <= 2);
        }

        #[test]
        fn prop_session_id_always_starts_with_prefix(_dummy: u8) {
            let repl = WasmRepl::new().unwrap();
            prop_assert!(repl.session_id().starts_with("session-"));
        }

        #[test]
        fn prop_clear_always_empties_history(count in 1usize..20) {
            let mut repl = WasmRepl::new().unwrap();
            for i in 0..count {
                let _ = repl.eval(&format!("{i}"));
            }
            repl.clear();
            prop_assert!(repl.get_history().is_empty());
        }

        #[test]
        fn prop_history_length_matches_eval_count(count in 1usize..50) {
            let mut repl = WasmRepl::new().unwrap();
            for i in 0..count {
                let _ = repl.eval(&format!("{i}"));
            }
            prop_assert_eq!(repl.get_history().len(), count);
        }
    }
}
