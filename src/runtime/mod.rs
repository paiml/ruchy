//! Runtime execution and REPL support
//!
//! This module provides the interactive REPL, runtime execution environment,
//! and actor system with supervision trees.

pub mod actor;
pub mod cache;
pub mod dataflow_debugger;
pub mod dataflow_ui;
pub mod grammar_coverage;
pub mod interpreter;
pub mod lazy;
pub mod observatory;
pub mod observatory_ui;
pub mod repl;
pub mod replay;
pub mod deterministic;
#[cfg(test)]
mod repl_function_tests;

// Export the unified REPL
pub use repl::{
    Repl, ReplConfig, ReplState, ReplMode, Checkpoint, Value,
    // Error Recovery System
    ErrorRecovery, RecoveryOption, RecoveryResult, DebugInfo
};

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

pub use observatory_ui::{DashboardConfig, DisplayMode, ObservatoryDashboard};

// Export dataflow debugger components
pub use dataflow_debugger::{
    DataflowDebugger, DataflowConfig, PipelineStage, StageType, StageStatus,
    Breakpoint, BreakpointCondition, BreakpointAction, MaterializedFrame, 
    StageMetrics, ExecutionEvent, StageDiff, ExportFormat,
};

pub use dataflow_ui::{DataflowUI, UIConfig};
