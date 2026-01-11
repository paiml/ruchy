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
//! let mut repl = Repl::new_with_config(config).expect("Repl creation should succeed");
//! repl.run().expect("REPL execution should succeed");
//! ```
//!
//! ```
//! use ruchy::runtime::{ActorSystem, Message, MessageValue};
//!
//! // Create an actor system
//! let mut system = ActorSystem::new();
//!
//! // Spawn an echo actor
//! let echo_ref = system.spawn_echo_actor("echo".to_string()).expect("Spawning actor should succeed");
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
pub mod actor_concurrent;
pub mod actor_runtime;
#[cfg(not(target_arch = "wasm32"))]
pub mod async_runtime;
pub mod audit; // Audit logging with entrenar integration
pub mod bytecode; // OPT-001: Bytecode VM Foundation
pub mod cache;
#[cfg(not(target_arch = "wasm32"))]
pub mod completion;
pub mod grammar_coverage;
pub mod interpreter;
#[cfg(test)]
pub mod interpreter_inline_tests; // Extracted tests from interpreter.rs
pub mod interpreter_types; // EXTREME TDD Round 52: InterpreterError, CallFrame extracted
pub mod interpreter_types_impl; // Class/struct/enum/actor definitions (re-export)
pub mod interpreter_types_actor; // Actor definition and instantiation
pub mod interpreter_types_struct; // Struct definition and instantiation
pub mod interpreter_types_class; // Class definition, instantiation, methods
pub mod interpreter_types_enum; // Enum and impl block definitions
pub mod interpreter_types_module; // Module expression evaluation
pub mod interpreter_methods; // Method dispatch for all value types (re-export)
pub mod interpreter_methods_string; // String/array method dispatch
pub mod interpreter_methods_dispatch; // Core method dispatch logic
#[cfg(not(target_arch = "wasm32"))]
pub mod interpreter_methods_html; // HTML document/element methods
pub mod interpreter_methods_instance; // Mutable object instance methods
pub mod interpreter_methods_actor; // Actor/struct/object methods
pub mod interpreter_dataframe; // DataFrame operations and methods
pub mod interpreter_control_flow; // Control flow: loops, match, assignments
pub mod interpreter_functions; // Function definitions, lambdas, and calls
pub mod interpreter_index; // Index access, field access, qualified names
pub mod module_loader; // PARSER-060: Module resolution and import system
pub mod object_helpers; // EXTREME TDD: RefCell-based mutable objects
pub mod value; // EXTREME TDD Round 52: Value type extracted from interpreter.rs
pub mod value_utils;
// Decomposed interpreter modules
pub mod builtin_init; // EXTREME TDD: Builtin functions initialization
pub mod eval_actor; // EXTREME TDD: Actor operations extracted for 100% coverage
pub mod builtins;
pub mod compilation; // EXTREME TDD: Direct-threaded interpreter compilation
pub mod eval_array;
pub mod eval_builtin;
pub mod eval_control_flow_new;
pub mod eval_data_structures;
pub mod eval_dataframe;
pub mod eval_dataframe_ops;
pub mod eval_display;
pub mod eval_expr;
pub mod eval_func;
pub mod eval_function;
pub mod eval_index; // EXTREME TDD: Index operations extracted for 100% coverage
pub mod eval_json; // EXTREME TDD: JSON operations extracted for 100% coverage
pub mod eval_html_methods; // HTTP-002-D: HTML parsing method support
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
pub mod type_feedback; // EXTREME TDD: JIT type feedback system (extracted from interpreter.rs)
pub mod value_format; // EXTREME TDD: Value formatting utilities (extracted from interpreter.rs)
pub mod gc_impl; // EXTREME TDD: Full GC implementation with tests
pub mod validation;
// pub mod interpreter_modules;  // Temporarily disabled - compilation errors
pub mod lazy;
pub mod pattern_matching;
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub mod repl; // New EXTREME Quality REPL
              // pub mod repl_legacy; // Old REPL (backup) - temporarily disabled for integration
              // pub mod repl_modules;  // Temporarily disabled - compilation errors
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub mod assessment;
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub mod deterministic;
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub mod magic;
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub mod repl_recording;
pub mod replay;
pub mod replay_converter;
// pub mod arena;  // Disabled - uses unsafe code
pub mod inspect;
pub mod safe_arena;
pub mod transaction;
// pub mod resource_eval;  // Temporarily disabled - causes duplicate impl
// Export the unified REPL
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub use repl::{Repl, ReplConfig};
// pub use repl_legacy::{ReplConfig, ReplState as LegacyReplState}; // Temporarily disabled
// Export interpreter components
pub use interpreter::Interpreter;
// Export interpreter types from dedicated module (EXTREME TDD Round 52)
pub use interpreter_types::{CallFrame, InterpreterError, InterpreterResult};
// Export Value types from dedicated module (EXTREME TDD Round 52)
pub use value::{DataFrameColumn, Value};
// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
// Export concurrent actor system
pub use actor_concurrent::{
    ActorState as ConcurrentActorState, ConcurrentActor, ConcurrentActorSystem, Envelope,
    SupervisionStrategy, SystemMessage, CONCURRENT_ACTOR_SYSTEM,
};
// Export assessment components
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
pub use assessment::{
    Assignment, AssignmentSetup, ExpectedBehavior, GradeReport, GradingEngine, GradingRubric,
    PlagiarismDetector, SecureSandbox, Task, TaskGrade, TestCase,
};
// Export magic commands
#[cfg(all(not(target_arch = "wasm32"), feature = "repl"))]
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
// Export replay-to-test converter
pub use replay_converter::{ConversionConfig, GeneratedTest, ReplayConverter, TestCategory};
// Export audit logging (entrenar integration)
pub use audit::{
    hash_chain_collector, ring_collector, stream_collector, EvalType, HashChainAuditCollector,
    ReplAuditCollector, ReplInputMode, ReplPath, RingAuditCollector, StreamAuditCollector,
    StreamFormat,
};

// EXTREME TDD Round 86: Comprehensive tests extracted to separate files
#[cfg(test)]
mod interpreter_tests;
#[cfg(test)]
mod eval_builtin_tests;

#[cfg(all(test, feature = "repl"))]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Sprint 4: Comprehensive runtime tests for coverage improvement
    // Requires `repl` feature since all tests use Repl

    #[test]
    fn test_repl_creation_and_basic_eval() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("1 + 1")
                .expect("operation should succeed in test"),
            "2"
        );
        assert_eq!(
            repl.eval("2 * 3")
                .expect("operation should succeed in test"),
            "6"
        );
        assert_eq!(
            repl.eval("10 - 5")
                .expect("operation should succeed in test"),
            "5"
        );
    }

    #[test]
    fn test_repl_variable_binding() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("let x = 42")
                .expect("operation should succeed in test"),
            "42"
        );
        assert_eq!(
            repl.eval("x").expect("operation should succeed in test"),
            "42"
        );
        assert_eq!(
            repl.eval("let y = x + 8")
                .expect("operation should succeed in test"),
            "50"
        );
        assert_eq!(
            repl.eval("y").expect("operation should succeed in test"),
            "50"
        );
    }

    #[test]
    fn test_repl_function_definition() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("fn add(a, b) { a + b }")
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("add(3, 4)")
                .expect("operation should succeed in test"),
            "7"
        );
        assert_eq!(
            repl.eval("add(10, 20)")
                .expect("operation should succeed in test"),
            "30"
        );
    }

    #[test]
    fn test_repl_if_expression() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("if true { 1 } else { 2 }")
                .expect("operation should succeed in test"),
            "1"
        );
        assert_eq!(
            repl.eval("if false { 1 } else { 2 }")
                .expect("operation should succeed in test"),
            "2"
        );
        assert_eq!(
            repl.eval("if 5 > 3 { \"yes\" } else { \"no\" }")
                .expect("operation should succeed in test"),
            "\"yes\""
        );
    }

    #[test]
    fn test_repl_list_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("[1, 2, 3]")
                .expect("operation should succeed in test"),
            "[1, 2, 3]"
        );
        assert_eq!(
            repl.eval("[]").expect("operation should succeed in test"),
            "[]"
        );
        assert_eq!(
            repl.eval("[1] + [2, 3]")
                .expect("operation should succeed in test"),
            "[1, 2, 3]"
        );
    }

    #[test]
    fn test_repl_for_loop() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("let mut sum = 0")
            .expect("operation should succeed in test");
        repl.eval("for i in 1..=5 { sum = sum + i }")
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("sum").expect("operation should succeed in test"),
            "15"
        );
    }

    #[test]
    fn test_repl_while_loop() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("let mut n = 0")
            .expect("operation should succeed in test");
        repl.eval("while n < 5 { n = n + 1 }")
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("n").expect("operation should succeed in test"),
            "5"
        );
    }

    #[test]
    fn test_repl_match_expression() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        let code = r#"
            match 2 {
                1 => "one",
                2 => "two",
                _ => "other"
            }
        "#;
        assert_eq!(
            repl.eval(code).expect("operation should succeed in test"),
            "\"two\""
        );
    }

    #[test]
    fn test_repl_lambda() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("let double = |x| x * 2")
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("double(21)")
                .expect("operation should succeed in test"),
            "42"
        );
    }

    #[test]
    fn test_repl_string_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("\"hello\" + \" world\"")
                .expect("operation should succeed in test"),
            "\"hello world\""
        );
        assert_eq!(
            repl.eval("\"test\"")
                .expect("operation should succeed in test"),
            "\"test\""
        );
    }

    #[test]
    fn test_repl_boolean_operations() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("true && true")
                .expect("operation should succeed in test"),
            "true"
        );
        assert_eq!(
            repl.eval("true || false")
                .expect("operation should succeed in test"),
            "true"
        );
        assert_eq!(
            repl.eval("!true")
                .expect("operation should succeed in test"),
            "false"
        );
    }

    #[test]
    fn test_repl_comparison_operators() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("5 > 3")
                .expect("operation should succeed in test"),
            "true"
        );
        assert_eq!(
            repl.eval("3 < 5")
                .expect("operation should succeed in test"),
            "true"
        );
        assert_eq!(
            repl.eval("5 == 5")
                .expect("operation should succeed in test"),
            "true"
        );
        assert_eq!(
            repl.eval("5 != 3")
                .expect("operation should succeed in test"),
            "true"
        );
    }

    #[test]
    fn test_repl_float_arithmetic() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert_eq!(
            repl.eval("3.5 + 1.5")
                .expect("operation should succeed in test"),
            "5.0"
        );
        assert_eq!(
            repl.eval("10.0 - 2.5")
                .expect("operation should succeed in test"),
            "7.5"
        );
        assert_eq!(
            repl.eval("2.5 * 2.0")
                .expect("operation should succeed in test"),
            "5.0"
        );
    }

    #[test]
    fn test_repl_error_handling() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        assert!(repl.eval("undefined_var").is_err());
        assert!(repl.eval("1 / 0").is_err());
        // Should recover after error
        assert_eq!(
            repl.eval("2 + 2")
                .expect("operation should succeed in test"),
            "4"
        );
    }

    #[test]
    fn test_repl_memory_tracking() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        let initial = repl.memory_used();
        assert_eq!(initial, 0);

        repl.eval("let x = [1, 2, 3, 4, 5]")
            .expect("operation should succeed in test");
        assert!(repl.memory_used() >= initial);

        let pressure = repl.memory_pressure();
        assert!((0.0..=1.0).contains(&pressure));
    }

    #[test]
    fn test_repl_checkpoint_restore() {
        use crate::runtime::replay::DeterministicRepl;

        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("let x = 10")
            .expect("operation should succeed in test");

        let checkpoint = DeterministicRepl::checkpoint(&repl);
        repl.eval("let x = 20")
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("x").expect("operation should succeed in test"),
            "20"
        );

        DeterministicRepl::restore(&mut repl, &checkpoint)
            .expect("operation should succeed in test");
        assert_eq!(
            repl.eval("x").expect("operation should succeed in test"),
            "10"
        );
    }

    #[test]
    fn test_repl_bindings_management() {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed in test");
        repl.eval("let a = 1")
            .expect("operation should succeed in test");
        repl.eval("let b = 2")
            .expect("operation should succeed in test");

        let bindings = repl.get_bindings();
        assert!(bindings.contains_key("a"));
        assert!(bindings.contains_key("b"));

        repl.clear_bindings();
        assert!(repl.get_bindings().is_empty());
    }

    #[test]
    fn test_value_types() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.15).to_string(), "3.15");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(
            Value::from_string("hello".to_string()).to_string(),
            "\"hello\""
        );
        assert_eq!(Value::Nil.to_string(), "nil");
    }

    #[test]
    fn test_value_list() {
        let list = Value::Array(Arc::from(
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].as_slice(),
        ));
        assert_eq!(list.to_string(), "[1, 2, 3]");

        let empty = Value::from_array(vec![]);
        assert_eq!(empty.to_string(), "[]");
    }

    #[test]
    fn test_value_tuple() {
        let tuple = Value::Tuple(Arc::from(
            vec![Value::Integer(1), Value::from_string("test".to_string())].as_slice(),
        ));
        assert_eq!(tuple.to_string(), "(1, \"test\")");
    }
}
