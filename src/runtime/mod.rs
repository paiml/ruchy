//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL, runtime execution environment,
//! and actor system with supervision trees for the Ruchy language.
//!
//! # Core Components
//!
//! ## REPL (Read-Eval-Print Loop)
//! Interactive development environment with:
//! - Line editing and history
//! - Tab completion
//! - Magic commands
//! - Session recording and replay
//!
//! ## Actor System
//! Concurrent programming model featuring:
//! - Message passing between actors
//! - Supervision trees for fault tolerance
//! - Actor observability and debugging
//! - Deadlock detection
//!
//! ## Runtime Execution
//! - Expression evaluation engine
//! - Pattern matching implementation
//! - Binary operations with proper semantics
//! - Memory-safe arena allocation
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::runtime::{Repl, ReplConfig};
//!
//! // Start an interactive REPL session
//! let config = ReplConfig::default();
//! let mut repl = Repl::new_with_config(config).unwrap();
//! repl.run().unwrap();
//! ```
//!
//! ```
//! use ruchy::runtime::{ActorSystem, Message, MessageValue};
//!
//! // Create an actor system
//! let mut system = ActorSystem::new();
//!
//! // Spawn an echo actor
//! let echo_ref = system.spawn_echo_actor("echo".to_string()).unwrap();
//!
//! // Send a message
//! let msg = Message::new(MessageValue::String("Hello".to_string()));
//! // Note: In real usage, you would handle the Result properly
//! ```
//!
//! # Features
//!
//! - **Interactive Development**: Full-featured REPL with completion
//! - **Concurrent Programming**: Actor model with supervision
//! - **Debugging Tools**: Observatory for system introspection
//! - **Memory Safety**: Arena allocation without unsafe code
//! - **Educational Tools**: Assessment and grading systems
pub mod actor;
pub mod async_runtime;
pub mod cache;
pub mod completion;
pub mod dataflow_debugger;
pub mod dataflow_ui;
pub mod grammar_coverage;
pub mod interpreter;
pub mod value_utils;
// Decomposed interpreter modules
pub mod builtin_init; // EXTREME TDD: Builtin functions initialization
pub mod builtins;
pub mod compilation; // EXTREME TDD: Direct-threaded interpreter compilation
pub mod eval_array;
pub mod eval_builtin;
pub mod eval_control_flow;
pub mod eval_control_flow_new;
pub mod eval_data_structures;
pub mod eval_dataframe;
pub mod eval_dataframe_ops;
pub mod eval_display;
pub mod eval_expr;
pub mod eval_func;
pub mod eval_function;
pub mod eval_literal;
pub mod eval_loops;
pub mod eval_method;
pub mod eval_method_dispatch;
pub mod eval_operations;
pub mod eval_pattern;
pub mod eval_pattern_match;
pub mod eval_string;
pub mod eval_string_interpolation;
pub mod eval_string_methods;
pub mod eval_try_catch; // EXTREME TDD: Try/catch error handling
pub mod gc;
pub mod gc_impl; // EXTREME TDD: Full GC implementation with tests
pub mod transformation;
pub mod validation;
// pub mod interpreter_modules;  // Temporarily disabled - compilation errors
pub mod lazy;
pub mod observatory;
pub mod observatory_ui;
pub mod pattern_matching;
pub mod repl; // New EXTREME Quality REPL
              // pub mod repl_legacy; // Old REPL (backup) - temporarily disabled for integration
              // pub mod repl_modules;  // Temporarily disabled - compilation errors
pub mod assessment;
pub mod deterministic;
pub mod magic;
pub mod repl_recording;
pub mod replay;
pub mod replay_converter;
// pub mod arena;  // Disabled - uses unsafe code
pub mod inspect;
pub mod safe_arena;
pub mod transaction;
// pub mod resource_eval;  // Temporarily disabled - causes duplicate impl
// Export the unified REPL
pub use repl::{Repl, ReplConfig};
// pub use repl_legacy::{ReplConfig, ReplState as LegacyReplState}; // Temporarily disabled
// Export interpreter components
pub use interpreter::{DataFrameColumn, Interpreter, InterpreterError, InterpreterResult, Value};
// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
// Export observatory components
pub use observatory::{
    ActorObservatory, ActorSnapshot, ActorState, DeadlockCycle, MessageFilter, MessageStatus,
    MessageTrace, ObservatoryConfig, SystemMetrics,
};
// Export assessment components
pub use assessment::{
    Assignment, AssignmentSetup, ExpectedBehavior, GradeReport, GradingEngine, GradingRubric,
    PlagiarismDetector, SecureSandbox, Task, TaskGrade, TestCase,
};
// Export magic commands
pub use magic::{MagicCommand, MagicRegistry, MagicResult, ProfileData, UnicodeExpander};
// Export inspection protocol
pub use inspect::{CompositeForm, DisplayForm, Inspect, InspectStyle, Inspector, OpaqueHandle};
// Export resource-bounded evaluation
pub use safe_arena::{SafeArena as Arena, TransactionalArena};
pub use transaction::{
    SavePoint, TransactionEvent, TransactionId, TransactionLog, TransactionMetadata,
    TransactionalState, Version, VersionedValue, MVCC,
};
// pub use resource_eval::{
//     CheckpointHandle, ResourceLimits, Sandbox,
// };
pub use observatory_ui::{DashboardConfig, DisplayMode, ObservatoryDashboard};
// Export dataflow debugger components
pub use dataflow_debugger::{
    Breakpoint, BreakpointAction, BreakpointCondition, DataflowConfig, DataflowDebugger,
    ExecutionEvent, ExportFormat, MaterializedFrame, PipelineStage, StageDiff, StageMetrics,
    StageStatus, StageType,
};
pub use dataflow_ui::{DataflowUI, UIConfig};
// Export replay-to-test converter
pub use replay_converter::{ConversionConfig, GeneratedTest, ReplayConverter, TestCategory};

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    // Sprint 4: Comprehensive runtime tests for coverage improvement

    #[test]
    fn test_repl_creation_and_basic_eval() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("1 + 1").unwrap(), "2");
        assert_eq!(repl.eval("2 * 3").unwrap(), "6");
        assert_eq!(repl.eval("10 - 5").unwrap(), "5");
    }

    #[test]
    fn test_repl_variable_binding() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("let x = 42").unwrap(), "42");
        assert_eq!(repl.eval("x").unwrap(), "42");
        assert_eq!(repl.eval("let y = x + 8").unwrap(), "50");
        assert_eq!(repl.eval("y").unwrap(), "50");
    }

    #[test]
    fn test_repl_function_definition() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("fn add(a, b) { a + b }").unwrap();
        assert_eq!(repl.eval("add(3, 4)").unwrap(), "7");
        assert_eq!(repl.eval("add(10, 20)").unwrap(), "30");
    }

    #[test]
    fn test_repl_if_expression() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("if true { 1 } else { 2 }").unwrap(), "1");
        assert_eq!(repl.eval("if false { 1 } else { 2 }").unwrap(), "2");
        assert_eq!(
            repl.eval("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap(),
            "\"yes\""
        );
    }

    #[test]
    fn test_repl_list_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("[1, 2, 3]").unwrap(), "[1, 2, 3]");
        assert_eq!(repl.eval("[]").unwrap(), "[]");
        assert_eq!(repl.eval("[1] + [2, 3]").unwrap(), "[1, 2, 3]");
    }

    #[test]
    fn test_repl_for_loop() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("let mut sum = 0").unwrap();
        repl.eval("for i in 1..=5 { sum = sum + i }").unwrap();
        assert_eq!(repl.eval("sum").unwrap(), "15");
    }

    #[test]
    fn test_repl_while_loop() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("let mut n = 0").unwrap();
        repl.eval("while n < 5 { n = n + 1 }").unwrap();
        assert_eq!(repl.eval("n").unwrap(), "5");
    }

    #[test]
    fn test_repl_match_expression() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let code = r#"
            match 2 {
                1 => "one",
                2 => "two",
                _ => "other"
            }
        "#;
        assert_eq!(repl.eval(code).unwrap(), "\"two\"");
    }

    #[test]
    fn test_repl_lambda() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("let double = |x| x * 2").unwrap();
        assert_eq!(repl.eval("double(21)").unwrap(), "42");
    }

    #[test]
    fn test_repl_string_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(
            repl.eval("\"hello\" + \" world\"").unwrap(),
            "\"hello world\""
        );
        assert_eq!(repl.eval("\"test\"").unwrap(), "\"test\"");
    }

    #[test]
    fn test_repl_boolean_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("true && true").unwrap(), "true");
        assert_eq!(repl.eval("true || false").unwrap(), "true");
        assert_eq!(repl.eval("!true").unwrap(), "false");
    }

    #[test]
    fn test_repl_comparison_operators() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("5 > 3").unwrap(), "true");
        assert_eq!(repl.eval("3 < 5").unwrap(), "true");
        assert_eq!(repl.eval("5 == 5").unwrap(), "true");
        assert_eq!(repl.eval("5 != 3").unwrap(), "true");
    }

    #[test]
    fn test_repl_float_arithmetic() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert_eq!(repl.eval("3.5 + 1.5").unwrap(), "5.0");
        assert_eq!(repl.eval("10.0 - 2.5").unwrap(), "7.5");
        assert_eq!(repl.eval("2.5 * 2.0").unwrap(), "5.0");
    }

    #[test]
    fn test_repl_error_handling() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        assert!(repl.eval("undefined_var").is_err());
        assert!(repl.eval("1 / 0").is_err());
        // Should recover after error
        assert_eq!(repl.eval("2 + 2").unwrap(), "4");
    }

    #[test]
    fn test_repl_memory_tracking() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let initial = repl.memory_used();
        assert_eq!(initial, 0);

        repl.eval("let x = [1, 2, 3, 4, 5]").unwrap();
        assert!(repl.memory_used() >= initial);

        let pressure = repl.memory_pressure();
        assert!((0.0..=1.0).contains(&pressure));
    }

    #[test]
    fn test_repl_checkpoint_restore() {
        use crate::runtime::replay::DeterministicRepl;

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("let x = 10").unwrap();

        let checkpoint = DeterministicRepl::checkpoint(&repl);
        repl.eval("let x = 20").unwrap();
        assert_eq!(repl.eval("x").unwrap(), "20");

        DeterministicRepl::restore(&mut repl, &checkpoint).unwrap();
        assert_eq!(repl.eval("x").unwrap(), "10");
    }

    #[test]
    fn test_repl_bindings_management() {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        repl.eval("let a = 1").unwrap();
        repl.eval("let b = 2").unwrap();

        let bindings = repl.get_bindings();
        assert!(bindings.contains_key("a"));
        assert!(bindings.contains_key("b"));

        repl.clear_bindings();
        assert!(repl.get_bindings().is_empty());
    }

    #[test]
    fn test_value_types() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(
            Value::from_string("hello".to_string()).to_string(),
            "\"hello\""
        );
        assert_eq!(Value::Nil.to_string(), "nil");
    }

    #[test]
    fn test_value_list() {
        let list = Value::Array(Rc::from(
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].as_slice(),
        ));
        assert_eq!(list.to_string(), "[1, 2, 3]");

        let empty = Value::from_array(vec![]);
        assert_eq!(empty.to_string(), "[]");
    }

    #[test]
    fn test_value_tuple() {
        let tuple = Value::Tuple(Rc::from(
            vec![Value::Integer(1), Value::from_string("test".to_string())].as_slice(),
        ));
        assert_eq!(tuple.to_string(), "(1, \"test\")");
    }
}
