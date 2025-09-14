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
pub mod binary_ops;
pub mod cache;
pub mod completion;
pub mod dataflow_debugger;
pub mod dataflow_ui;
pub mod grammar_coverage;
pub mod interpreter;
// pub mod interpreter_modules;  // Temporarily disabled - compilation errors
pub mod lazy;
pub mod pattern_matching;
pub mod observatory;
pub mod observatory_ui;
pub mod repl;
// pub mod repl_modules;  // Temporarily disabled - compilation errors
pub mod repl_recording;
pub mod replay;
pub mod replay_converter;
pub mod deterministic;
pub mod assessment;
pub mod magic;
// pub mod arena;  // Disabled - uses unsafe code
pub mod safe_arena;
pub mod transaction;
pub mod inspect;
// pub mod resource_eval;  // Temporarily disabled - causes duplicate impl
// Export the unified REPL
pub use repl::{Repl, ReplConfig, ReplState, Value};
// Export interpreter components
pub use interpreter::{
    Interpreter, InterpreterError, InterpreterResult, Value as InterpreterValue,
};
// Export actor system components
pub use actor::{
    ActorBehavior, ActorContext, ActorId, ActorRef, ActorSystem, EchoActor, Message, MessageValue,
    SupervisorActor, SupervisorDirective,
};
// Export observatory components
pub use observatory::{
    ActorObservatory, ActorSnapshot, ActorState, DeadlockCycle, MessageFilter, MessageTrace,
    MessageStatus, ObservatoryConfig, SystemMetrics,
};
// Export assessment components
pub use assessment::{
    Assignment, AssignmentSetup, Task, TestCase, ExpectedBehavior,
    GradingEngine, GradeReport, TaskGrade, GradingRubric,
    PlagiarismDetector, SecureSandbox,
};
// Export magic commands
pub use magic::{
    MagicRegistry, MagicResult, MagicCommand, UnicodeExpander, ProfileData,
};
// Export inspection protocol
pub use inspect::{
    Inspect, Inspector, InspectStyle, DisplayForm, CompositeForm, OpaqueHandle,
};
// Export resource-bounded evaluation
pub use safe_arena::{
    SafeArena as Arena, TransactionalArena,
};
pub use transaction::{
    TransactionalState, TransactionId, TransactionMetadata, SavePoint,
    TransactionEvent, TransactionLog, MVCC, Version, VersionedValue,
};
// pub use resource_eval::{
//     CheckpointHandle, ResourceLimits, Sandbox,
// };
pub use observatory_ui::{DashboardConfig, DisplayMode, ObservatoryDashboard};
// Export dataflow debugger components
pub use dataflow_debugger::{
    DataflowDebugger, DataflowConfig, PipelineStage, StageType, StageStatus,
    Breakpoint, BreakpointCondition, BreakpointAction, MaterializedFrame, 
    StageMetrics, ExecutionEvent, StageDiff, ExportFormat,
};
pub use dataflow_ui::{DataflowUI, UIConfig};
// Export replay-to-test converter
pub use replay_converter::{
    ReplayConverter, ConversionConfig, GeneratedTest, TestCategory,
};
