//! REPL Output Types
//!
//! Data structures for WASM REPL evaluation results.

use serde::{Deserialize, Serialize};

/// Output from REPL evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplOutput {
    /// Whether evaluation succeeded
    pub success: bool,
    /// Display value (if any)
    pub display: Option<String>,
    /// Type information
    pub type_info: Option<String>,
    /// Generated Rust code
    pub rust_code: Option<String>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Timing information
    pub timing: TimingInfo,
}

/// Timing information for REPL evaluation phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Time spent parsing (milliseconds)
    pub parse_ms: f64,
    /// Time spent type checking (milliseconds)
    pub typecheck_ms: f64,
    /// Time spent evaluating (milliseconds)
    pub eval_ms: f64,
    /// Total time (milliseconds)
    pub total_ms: f64,
}

impl ReplOutput {
    /// Create a successful output
    pub fn success(display: String, timing: TimingInfo) -> Self {
        Self {
            success: true,
            display: Some(display),
            type_info: Some("Any".to_string()),
            rust_code: None,
            error: None,
            timing,
        }
    }

    /// Create a parse error output
    pub fn parse_error(error: String, timing: TimingInfo) -> Self {
        Self {
            success: false,
            display: None,
            type_info: None,
            rust_code: None,
            error: Some(format!("Parse error: {error}")),
            timing,
        }
    }

    /// Create a runtime error output
    pub fn runtime_error(error: String, timing: TimingInfo) -> Self {
        Self {
            success: false,
            display: None,
            type_info: None,
            rust_code: None,
            error: Some(format!("Runtime error: {error}")),
            timing,
        }
    }
}

impl TimingInfo {
    /// Create new timing info
    pub fn new(parse_ms: f64, typecheck_ms: f64, eval_ms: f64, total_ms: f64) -> Self {
        Self {
            parse_ms,
            typecheck_ms,
            eval_ms,
            total_ms,
        }
    }

    /// Create timing info with just parse and total (for parse errors)
    pub fn parse_only(parse_ms: f64, total_ms: f64) -> Self {
        Self {
            parse_ms,
            typecheck_ms: 0.0,
            eval_ms: 0.0,
            total_ms,
        }
    }

    /// Create timing info for eval phase
    pub fn with_eval(parse_ms: f64, eval_ms: f64, total_ms: f64) -> Self {
        Self {
            parse_ms,
            typecheck_ms: 0.0,
            eval_ms,
            total_ms,
        }
    }
}

impl Default for TimingInfo {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ReplOutput Tests =====

    #[test]
    fn test_repl_output_success() {
        let timing = TimingInfo::default();
        let output = ReplOutput::success("42".to_string(), timing);
        assert!(output.success);
        assert_eq!(output.display, Some("42".to_string()));
        assert!(output.error.is_none());
    }

    #[test]
    fn test_repl_output_parse_error() {
        let timing = TimingInfo::parse_only(1.0, 1.0);
        let output = ReplOutput::parse_error("unexpected token".to_string(), timing);
        assert!(!output.success);
        assert!(output.display.is_none());
        assert!(output.error.unwrap().contains("Parse error"));
    }

    #[test]
    fn test_repl_output_runtime_error() {
        let timing = TimingInfo::with_eval(1.0, 0.5, 1.5);
        let output = ReplOutput::runtime_error("division by zero".to_string(), timing);
        assert!(!output.success);
        assert!(output.error.unwrap().contains("Runtime error"));
    }

    #[test]
    fn test_repl_output_debug() {
        let output = ReplOutput {
            success: true,
            display: None,
            type_info: None,
            rust_code: None,
            error: None,
            timing: TimingInfo::default(),
        };
        let debug = format!("{:?}", output);
        assert!(debug.contains("ReplOutput"));
    }

    #[test]
    fn test_repl_output_clone() {
        let output = ReplOutput::success("test".to_string(), TimingInfo::default());
        let cloned = output.clone();
        assert_eq!(output.success, cloned.success);
        assert_eq!(output.display, cloned.display);
    }

    #[test]
    fn test_repl_output_serialize_deserialize() {
        let output = ReplOutput {
            success: true,
            display: Some("hello".to_string()),
            type_info: Some("String".to_string()),
            rust_code: None,
            error: None,
            timing: TimingInfo::new(1.5, 2.5, 3.5, 7.5),
        };
        let json = serde_json::to_string(&output).expect("serialize");
        let decoded: ReplOutput = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(output.success, decoded.success);
        assert_eq!(output.display, decoded.display);
        assert_eq!(output.type_info, decoded.type_info);
    }

    #[test]
    fn test_repl_output_with_all_fields() {
        let output = ReplOutput {
            success: true,
            display: Some("result".to_string()),
            type_info: Some("i64".to_string()),
            rust_code: Some("fn main() { println!(\"42\"); }".to_string()),
            error: None,
            timing: TimingInfo::new(0.5, 1.0, 0.3, 1.8),
        };
        let json = serde_json::to_string(&output).expect("serialize");
        assert!(json.contains("result"));
        assert!(json.contains("i64"));
        assert!(json.contains("fn main"));
    }

    #[test]
    fn test_repl_output_with_error_field() {
        let output = ReplOutput {
            success: false,
            display: None,
            type_info: None,
            rust_code: None,
            error: Some("Type mismatch: expected i64, got String".to_string()),
            timing: TimingInfo::new(0.1, 0.2, 0.0, 0.3),
        };
        let json = serde_json::to_string(&output).expect("serialize");
        let decoded: ReplOutput = serde_json::from_str(&json).expect("deserialize");
        assert!(!decoded.success);
        assert!(decoded.error.unwrap().contains("Type mismatch"));
    }

    // ===== TimingInfo Tests =====

    #[test]
    fn test_timing_info_new() {
        let timing = TimingInfo::new(1.0, 2.0, 3.0, 6.0);
        assert!((timing.parse_ms - 1.0).abs() < f64::EPSILON);
        assert!((timing.typecheck_ms - 2.0).abs() < f64::EPSILON);
        assert!((timing.eval_ms - 3.0).abs() < f64::EPSILON);
        assert!((timing.total_ms - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timing_info_parse_only() {
        let timing = TimingInfo::parse_only(1.5, 1.5);
        assert!((timing.parse_ms - 1.5).abs() < f64::EPSILON);
        assert!((timing.typecheck_ms - 0.0).abs() < f64::EPSILON);
        assert!((timing.eval_ms - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timing_info_with_eval() {
        let timing = TimingInfo::with_eval(1.0, 2.0, 3.0);
        assert!((timing.parse_ms - 1.0).abs() < f64::EPSILON);
        assert!((timing.eval_ms - 2.0).abs() < f64::EPSILON);
        assert!((timing.total_ms - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timing_info_default() {
        let timing = TimingInfo::default();
        assert!((timing.parse_ms - 0.0).abs() < f64::EPSILON);
        assert!((timing.total_ms - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timing_info_debug() {
        let timing = TimingInfo::new(1.0, 2.0, 3.0, 6.0);
        let debug = format!("{:?}", timing);
        assert!(debug.contains("TimingInfo"));
    }

    #[test]
    fn test_timing_info_clone() {
        let timing = TimingInfo::new(1.0, 2.0, 3.0, 6.0);
        let cloned = timing.clone();
        assert!((timing.parse_ms - cloned.parse_ms).abs() < f64::EPSILON);
    }

    #[test]
    fn test_timing_info_serialize_deserialize() {
        let timing = TimingInfo::new(1.5, 2.5, 3.5, 7.5);
        let json = serde_json::to_string(&timing).expect("serialize");
        let decoded: TimingInfo = serde_json::from_str(&json).expect("deserialize");
        assert!((timing.parse_ms - decoded.parse_ms).abs() < f64::EPSILON);
        assert!((timing.total_ms - decoded.total_ms).abs() < f64::EPSILON);
    }

    // ===== Property Tests =====

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(500))]

            /// Property: ReplOutput serialization roundtrips
            #[test]
            fn prop_repl_output_roundtrip(
                success in proptest::bool::ANY,
                display in proptest::option::of("[a-z]{1,30}")
            ) {
                let output = ReplOutput {
                    success,
                    display,
                    type_info: None,
                    rust_code: None,
                    error: None,
                    timing: TimingInfo::default(),
                };
                let json = serde_json::to_string(&output).unwrap();
                let decoded: ReplOutput = serde_json::from_str(&json).unwrap();
                prop_assert_eq!(output.success, decoded.success);
                prop_assert_eq!(output.display, decoded.display);
            }

            /// Property: TimingInfo always has non-negative values
            #[test]
            fn prop_timing_info_non_negative(
                parse in 0.0f64..1000.0,
                typecheck in 0.0f64..1000.0,
                eval in 0.0f64..1000.0
            ) {
                let timing = TimingInfo::new(parse, typecheck, eval, parse + typecheck + eval);
                prop_assert!(timing.parse_ms >= 0.0);
                prop_assert!(timing.typecheck_ms >= 0.0);
                prop_assert!(timing.eval_ms >= 0.0);
                prop_assert!(timing.total_ms >= 0.0);
            }

            /// Property: TimingInfo clone is identical
            #[test]
            fn prop_timing_clone_identical(
                parse in 0.0f64..100.0,
                eval in 0.0f64..100.0
            ) {
                let timing = TimingInfo::with_eval(parse, eval, parse + eval);
                let cloned = timing.clone();
                prop_assert!((timing.parse_ms - cloned.parse_ms).abs() < f64::EPSILON);
                prop_assert!((timing.eval_ms - cloned.eval_ms).abs() < f64::EPSILON);
            }
        }
    }
}
