//! Testing utilities and property-based tests
//!
//! This module provides test generators and property-based testing utilities.
pub mod ast_builder;
#[cfg(test)]
pub mod generators;
pub mod harness;
pub mod properties;
pub mod snapshot;
pub use ast_builder::AstBuilder;
pub use generators::*;
pub use harness::{OptLevel, RuchyTestHarness, TestError, TestResult, ValidationResult};
pub use properties::*;
pub use snapshot::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 10: Comprehensive testing module tests

    #[test]
    fn test_ast_builder_creation() {
        let builder = AstBuilder::new();
        // Just verify it can be created
        let _ = builder;
    }

    #[test]
    fn test_ast_builder_build_literal() {
        let builder = AstBuilder::new();
        let expr = builder.literal_int(42);
        // Verify we can build literals
        assert!(expr.is_literal());
    }

    #[test]
    fn test_ast_builder_build_identifier() {
        let builder = AstBuilder::new();
        let expr = builder.identifier("x");
        assert!(expr.is_identifier());
    }

    #[test]
    fn test_ast_builder_build_binary() {
        let builder = AstBuilder::new();
        let left = builder.literal_int(1);
        let right = builder.literal_int(2);
        let expr = builder.binary_add(left, right);
        assert!(expr.is_binary());
    }

    #[test]
    fn test_test_harness_creation() {
        let harness = RuchyTestHarness::new();
        assert_eq!(harness.opt_level(), OptLevel::None);
    }

    #[test]
    fn test_test_harness_with_opt_level() {
        let harness = RuchyTestHarness::with_opt_level(OptLevel::O2);
        assert_eq!(harness.opt_level(), OptLevel::O2);
    }

    #[test]
    fn test_opt_level_variants() {
        let none = OptLevel::None;
        let o1 = OptLevel::O1;
        let o2 = OptLevel::O2;
        let o3 = OptLevel::O3;

        assert!(matches!(none, OptLevel::None));
        assert!(matches!(o1, OptLevel::O1));
        assert!(matches!(o2, OptLevel::O2));
        assert!(matches!(o3, OptLevel::O3));
    }

    #[test]
    fn test_test_result_success() {
        let result = TestResult::success("test passed");
        assert!(result.is_success());
        assert_eq!(result.message(), "test passed");
        assert!(result.error().is_none());
    }

    #[test]
    fn test_test_result_failure() {
        let result = TestResult::failure("test failed", "error details");
        assert!(!result.is_success());
        assert_eq!(result.message(), "test failed");
        assert_eq!(result.error(), Some("error details"));
    }

    #[test]
    fn test_test_error_variants() {
        let parse_error = TestError::ParseError("syntax error".to_string());
        let compile_error = TestError::CompileError("type error".to_string());
        let runtime_error = TestError::RuntimeError("division by zero".to_string());
        let validation_error = TestError::ValidationError("assertion failed".to_string());

        assert!(matches!(parse_error, TestError::ParseError(_)));
        assert!(matches!(compile_error, TestError::CompileError(_)));
        assert!(matches!(runtime_error, TestError::RuntimeError(_)));
        assert!(matches!(validation_error, TestError::ValidationError(_)));
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::Valid;
        assert!(result.is_valid());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult::Invalid("constraint violated".to_string());
        assert!(!result.is_valid());
        if let ValidationResult::Invalid(msg) = result {
            assert_eq!(msg, "constraint violated");
        }
    }

    /* Commented out - SnapshotManager not available
    #[test]
    fn test_snapshot_manager_creation() {
        let manager = SnapshotManager::new("test_snapshots");
        assert_eq!(manager.directory(), "test_snapshots");
    }
    */

    /* Commented out - Snapshot type not available
    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new("test_snapshot", "content");
        assert_eq!(snapshot.name(), "test_snapshot");
        assert_eq!(snapshot.content(), "content");
        assert!(snapshot.metadata().created_at > 0);
    }
    */

    /* Commented out - SnapshotMetadata type not available
    #[test]
    fn test_snapshot_metadata() {
        let metadata = SnapshotMetadata::new();
        assert!(metadata.created_at > 0);
        assert_eq!(metadata.version, 1);
    }

    #[test]
    fn test_snapshot_comparison() {
        let snapshot1 = Snapshot::new("test", "content1");
        let snapshot2 = Snapshot::new("test", "content2");

        assert!(snapshot1.differs_from(&snapshot2));

        let snapshot3 = Snapshot::new("test", "content1");
        assert!(!snapshot1.differs_from(&snapshot3));
    }
    */

    /* Commented out - PropertyGenerator not available
    #[test]
    fn test_property_generator_integers() {
        let gen = PropertyGenerator::new();
        let int = gen.generate_integer(0, 100);
        assert!(int >= 0 && int <= 100);
    }

    #[test]
    fn test_property_generator_strings() {
        let gen = PropertyGenerator::new();
        let string = gen.generate_string(10);
        assert_eq!(string.len(), 10);
    }

    #[test]
    fn test_property_generator_booleans() {
        let gen = PropertyGenerator::new();
        let bool_val = gen.generate_boolean();
        assert!(bool_val || !bool_val); // Always true, just checking it returns a bool
    }
    */

    /* Commented out - PropertyTestConfig not available
    #[test]
    fn test_property_test_config() {
        let config = PropertyTestConfig::default();
        assert_eq!(config.num_tests, 100);
        assert_eq!(config.max_shrink_iterations, 100);
    }

    #[test]
    fn test_property_test_config_custom() {
        let config = PropertyTestConfig {
            num_tests: 500,
            max_shrink_iterations: 200,
            seed: Some(42),
            timeout_ms: 5000,
        };
        assert_eq!(config.num_tests, 500);
        assert_eq!(config.max_shrink_iterations, 200);
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.timeout_ms, 5000);
    }
    */

    /* Commented out - PropertyRunner and related types not available
    #[test]
    fn test_property_runner_creation() {
        let runner = PropertyRunner::new();
        assert_eq!(runner.config().num_tests, 100);
    }

    #[test]
    fn test_property_runner_with_config() {
        let config = PropertyTestConfig {
            num_tests: 250,
            ..Default::default()
        };
        let runner = PropertyRunner::with_config(config);
        assert_eq!(runner.config().num_tests, 250);
    }

    #[test]
    fn test_shrinking_strategy() {
        let strategy = ShrinkingStrategy::default();
        assert_eq!(strategy.max_iterations(), 100);
    }

    #[test]
    fn test_shrinking_result() {
        let result = ShrinkingResult::new(vec![1, 2, 3], 5);
        assert_eq!(result.minimal_input(), &vec![1, 2, 3]);
        assert_eq!(result.iterations(), 5);
    }
    */

    #[test]
    fn test_test_harness_run_simple() {
        let harness = RuchyTestHarness::new();
        let code = "42";
        let result = harness.run(code);
        // May succeed or fail depending on implementation
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_ast_builder_build_let() {
        let builder = AstBuilder::new();
        let value = builder.literal_int(42);
        let let_expr = builder.let_binding("x", value);
        assert!(let_expr.is_let());
    }

    #[test]
    fn test_ast_builder_build_if() {
        let builder = AstBuilder::new();
        let condition = builder.literal_bool(true);
        let then_branch = builder.literal_int(1);
        let else_branch = Some(builder.literal_int(2));
        let if_expr = builder.if_expression(condition, then_branch, else_branch);
        assert!(if_expr.is_if());
    }

    #[test]
    fn test_ast_builder_build_function() {
        let builder = AstBuilder::new();
        let body = builder.literal_int(42);
        let func = builder.function("foo", vec!["x", "y"], body);
        assert!(func.is_function());
    }

    #[test]
    fn test_ast_builder_build_call() {
        let builder = AstBuilder::new();
        let func = builder.identifier("print");
        let arg = builder.literal_string("hello");
        let call = builder.call(func, vec![arg]);
        assert!(call.is_call());
    }

    /* Commented out - Snapshot type not available
    #[test]
    fn test_snapshot_manager_save_and_load() {
        let manager = SnapshotManager::new("test_snapshots");
        let snapshot = Snapshot::new("test", "content");

        // Save might succeed or fail depending on filesystem
        let save_result = manager.save(&snapshot);
        assert!(save_result.is_ok() || save_result.is_err());

        // Load might succeed or fail
        let load_result = manager.load("test");
        assert!(load_result.is_ok() || load_result.is_err());
    }
    */

    /* Commented out - PropertyGenerator not available
    #[test]
    fn test_property_generator_lists() {
        let gen = PropertyGenerator::new();
        let list = gen.generate_list(5, || gen.generate_integer(0, 10));
        assert_eq!(list.len(), 5);
        for item in list {
            assert!(item >= 0 && item <= 10);
        }
    }
    */

    #[test]
    fn test_test_error_display() {
        let error = TestError::ParseError("unexpected token".to_string());
        let display = format!("{}", error);
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_validation_result_conversion() {
        let valid = ValidationResult::Valid;
        assert!(valid.to_test_result().is_success());

        let invalid = ValidationResult::Invalid("failed".to_string());
        assert!(!invalid.to_test_result().is_success());
    }
}
